use crate::{AppState, models::auth::Claims};
use axum::{
  extract::{FromRequestParts, Request, State},
  http::{StatusCode, header, request::Parts},
  middleware::Next,
  response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use redis::AsyncCommands;

fn decode_claims(token: &str, secret: &str) -> Result<Claims, StatusCode> {
  decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_ref()),
    &Validation::default(),
  )
  .map(|data| data.claims)
  .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(
  State(state): State<AppState>,
  mut req: Request,
  next: Next,
) -> Result<Response, StatusCode> {
  let raw_token = req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "))
    .ok_or(StatusCode::UNAUTHORIZED)?;

  // Query Redis to see if the token is on the logout blocklist
  let mut redis_conn = state.redis_connection.clone();
  let blocklist_key = format!("blocklist:{}", raw_token);

  let is_blocked: bool = redis_conn
    .exists(&blocklist_key)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  if is_blocked {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let claims: Claims =
    decode_claims(raw_token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

  // Inject claims into request extensions for subsequent route handlers
  req.extensions_mut().insert(claims);

  // Continue down the router chain
  Ok(next.run(req).await)
}

pub struct AuthenticatedUser(pub Claims);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
  S: Send + Sync,
{
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    // Extract the Claims that were inserted by the middleware
    parts
      .extensions
      .get::<Claims>()
      .cloned()
      .map(AuthenticatedUser)
      .ok_or(StatusCode::UNAUTHORIZED)
  }
}
