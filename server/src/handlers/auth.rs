use argon2::{
  Argon2,
  password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng as Osrng,
  },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use redis::AsyncCommands;
use std::env;
use uuid::Uuid;

use crate::{
  AppState,
  errors::AppError,
  infra::database::{
    auth::get_refresh_token,
    user::{create_user, find_user_by_email, find_user_by_id},
  },
  models::{
    auth::{AuthBody, AuthPayload, Claims, RefreshToken},
    user::UserResponse,
  },
};

pub async fn refresh_token(
  State(state): State<AppState>,
  Json(payload): Json<RefreshToken>,
) -> Result<impl IntoResponse, AppError> {
  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", payload.refresh_token);

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

      token_row.user_id
    }
  };

  let user = find_user_by_id(&state.pool, user_id)
    .await
    .map_err(|_| AppError::Internal("Database error"))?
    .ok_or(AppError::Unauthorized("User not found"))?;

  let token = generate_jwt(user.id, user.email)?;

  Ok(Json(AuthBody {
    access_token: token,
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

  Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
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

  let token = generate_jwt(user.id, user.email)?;

  Ok(Json(AuthBody {
    access_token: token,
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
