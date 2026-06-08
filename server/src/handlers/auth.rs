use argon2::{
  Argon2,
  password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng as Osrng,
  },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::{Rng, rng};
use redis::AsyncCommands;
use std::env;
use uuid::Uuid;

use crate::{
  AppState,
  errors::AppError,
  infra::database::{
    auth::{get_refresh_token, revoke_refresh_token, save_refresh_token},
    user::{create_user, find_user_by_email, find_user_by_id},
  },
  models::{
    auth::{AuthBody, AuthPayload, Claims, RefreshToken},
    user::UserResponse,
  },
};

pub async fn logout(
  State(state): State<AppState>,
  Json(payload): Json<RefreshToken>,
) -> Result<impl IntoResponse, AppError> {
  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", payload.refresh_token);

  // Remove from Redis
  let _: () = redis_conn
    .del(&redis_key)
    .await
    .map_err(|_| AppError::Internal("Redis error"))?;

  // Remove from Postgres
  revoke_refresh_token(&state.pool, &payload.refresh_token)
    .await
    .map_err(|_| AppError::Internal("Database error"))?;

  Ok(StatusCode::OK)
}

async fn generate_refresh_token(state: AppState, user_id: Uuid) -> Result<String, AppError> {
  // Generate a new refresh token,
  let mut bytes = [0u8; 32];
  rng().fill_bytes(&mut bytes);
  let refresh_token = hex::encode(bytes);

  // Save to Postgres
  let expires_at = save_refresh_token(&state.pool, user_id, &refresh_token, 7)
    .await
    .map_err(|_| AppError::Internal("Database error"))?;

  // Save to Redis (using TTL from days)
  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", refresh_token);
  let ttl = (expires_at - Utc::now()).num_seconds() as u64;

  let _: () = redis_conn
    .set_ex(&redis_key, user_id.to_string(), ttl)
    .await
    .map_err(|_| AppError::Internal("Redis error"))?;

  Ok(refresh_token)
}

pub async fn refresh_token(
  State(state): State<AppState>,
  Json(payload): Json<RefreshToken>,
) -> Result<impl IntoResponse, AppError> {
  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", payload.refresh_token);

  // Try Redis first
  let user_id: Uuid = match redis_conn.get::<_, Option<String>>(&redis_key).await {
    Ok(Some(id_str)) => {
      Uuid::parse_str(&id_str).map_err(|_| AppError::Internal("Invalid ID format in cache"))?
    }
    _ => {
      // Fallback to Postgres
      let token_row = get_refresh_token(&state.pool, &payload.refresh_token)
        .await
        .map_err(|_| AppError::Internal("Database error"))?
        .ok_or(AppError::Unauthorized("Invalid refresh token"))?;

      if token_row.expires_at < Utc::now().naive_utc() {
        return Err(AppError::Unauthorized("Refresh token expired"));
      }

      // Sync back to Redis for faster subsequent access
      let _: () = redis_conn
        .set_ex(&redis_key, token_row.user_id.to_string(), 7 * 24 * 60 * 60)
        .await
        .map_err(|_| AppError::Internal("Redis error"))?;

      token_row.user_id
    }
  };

  let user = find_user_by_id(&state.pool, user_id)
    .await
    .map_err(|_| AppError::Internal("Database error"))?
    .ok_or(AppError::Unauthorized("User not found"))?;

  let access_token = generate_jwt(user.id, user.email)?;

  Ok(Json(AuthBody {
    access_token,
    refresh_token: payload.refresh_token,
    token_type: "Bearer".to_string(),
  }))
}

pub async fn signup(
  State(state): State<AppState>,
  Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
  let salt = SaltString::generate(&mut Osrng);
  let argon2 = Argon2::default();

  let password_hash = argon2
    .hash_password(payload.password.as_bytes(), &salt)
    .map_err(|_| AppError::Internal("Error hashing password"))?
    .to_string();

  let user = create_user(&state.pool, &payload.email, &password_hash)
    .await
    .map_err(|e| {
      e.as_database_error()
        .filter(|e| e.is_unique_violation())
        .map(|_| AppError::Conflict("User already exists"))
        .unwrap_or(AppError::Internal("Internal server error"))
    })?;

  let access_token = generate_jwt(user.id, user.email.clone())?;
  let refresh_token: String = generate_refresh_token(state, user.id).await?;

  Ok((
    StatusCode::CREATED,
    Json(AuthBody {
      access_token,
      refresh_token,
      token_type: "Bearer".to_string(),
    }),
  ))
}

pub async fn login(
  State(state): State<AppState>,
  Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
  let user = find_user_by_email(&state.pool, &payload.email)
    .await
    .map_err(|_| AppError::Internal("Internal server error"))?
    .ok_or(AppError::Unauthorized("Invalid credentials"))?;

  let parsed_hash = PasswordHash::new(&user.hash_password)
    .map_err(|_| AppError::Internal("Invalid password hash format in database"))?;

  let is_valid = Argon2::default()
    .verify_password(payload.password.as_bytes(), &parsed_hash)
    .is_ok();

  if !is_valid {
    return Err(AppError::Unauthorized("Invalid credentials"));
  }

  let access_token = generate_jwt(user.id, user.email.clone())?;
  let refresh_token: String = generate_refresh_token(state, user.id).await?;

  Ok(Json(AuthBody {
    access_token,
    refresh_token,
    token_type: "Bearer".to_string(),
  }))
}

fn generate_jwt(user_id: Uuid, email: String) -> Result<String, AppError> {
  let secret = env::var("JWT_SECRET").map_err(|_| AppError::Internal("JWT_SECRET not set"))?;

  let exp = Utc::now()
    .checked_add_signed(Duration::hours(24))
    .expect("valid timestamp")
    .timestamp() as usize;

  let claims = Claims {
    sub: user_id,
    email,
    exp,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_ref()),
  )
  .map_err(|_| AppError::Internal("Error generating token"))
}
