use argon2::{
  Argon2,
  password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng as Osrng,
  },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;
use uuid::Uuid;

use crate::{
  AppState,
  errors::AppError,
  infra::database::user::{create_user, find_user_by_email},
  models::auth::{AuthBody, AuthPayload, Claims},
  models::user::UserResponse,
};

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
