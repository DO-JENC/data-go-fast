use crate::models::auth::Claims;
use axum::{
  extract::{FromRequestParts, Request},
  http::{StatusCode, header, request::Parts},
  middleware::Next,
  response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;

fn jwt_secret() -> Result<String, StatusCode> {
  env::var("JWT_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn decode_claims(token: &str, secret: &str) -> Result<Claims, StatusCode> {
  decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_ref()),
    &Validation::default(),
  )
  .map(|data| data.claims)
  .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
  let claims = {
    let token = req
      .headers()
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.strip_prefix("Bearer "))
      .ok_or(StatusCode::UNAUTHORIZED)?;

    decode_claims(token, &jwt_secret()?)?
  };

  req.extensions_mut().insert(claims);
  Ok(next.run(req).await)
}

// Extractor
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
