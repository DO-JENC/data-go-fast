use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
  pub sub: Uuid,
  pub email: String,
  pub exp: usize,
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
