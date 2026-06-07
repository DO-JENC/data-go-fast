use crate::models::user::User;
use sqlx::PgPool;

pub async fn create_user(
  pool: &PgPool,
  email: &str,
  hash_password: &str,
) -> Result<User, sqlx::Error> {
  let user = sqlx::query_as::<_, User>(
    "INSERT INTO users (email, hash_password) VALUES ($1, $2) RETURNING id, email, hash_password",
  )
  .bind(email)
  .bind(hash_password)
  .fetch_one(pool)
  .await?;

  Ok(user)
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
  let user =
    sqlx::query_as::<_, User>("SELECT id, email, hash_password FROM users WHERE email = $1")
      .bind(email)
      .fetch_optional(pool)
      .await?;

  Ok(user)
}

pub async fn find_user_by_id(pool: &PgPool, id: uuid::Uuid) -> Result<Option<User>, sqlx::Error> {
  let user = sqlx::query_as::<_, User>("SELECT id, email, hash_password FROM users WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

  Ok(user)
}
