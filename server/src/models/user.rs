use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
  pub id: Uuid,
  pub email: String,
  pub hash_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
  pub id: Uuid,
  pub email: String,
}

impl From<User> for UserResponse {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      email: user.email,
    }
  }
}
