use crate::infra::database::user::create_user;
use crate::models::user::{SignupRequest, UserResponse};
use argon2::{
  Argon2,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;

pub async fn signup(
  State(pool): State<PgPool>,
  Json(payload): Json<SignupRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, String)> {
  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();

  let password_hash = argon2
    .hash_password(payload.password.as_bytes(), &salt)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .to_string();

  let user = create_user(&pool, &payload.email, &password_hash)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        return (StatusCode::CONFLICT, "User already exists".to_string());
      }
      (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

  Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}
