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
