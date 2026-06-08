use axum::{
  extract::FromRequestParts,
  http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
  pub sub: Uuid,
  pub email: String,
  pub exp: usize,
}

#[derive(sqlx::FromRow)]
pub struct RefreshTokenRow {
  pub user_id: Uuid,
  pub expires_at: chrono::NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBody {
  pub access_token: String,
  pub token_type: String,
}

pub struct AuthenticatedUser(pub Claims);

impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    parts
      .extensions
      .get::<Claims>()
      .cloned()
      .map(AuthenticatedUser)
      .ok_or(StatusCode::UNAUTHORIZED)
  }
}
