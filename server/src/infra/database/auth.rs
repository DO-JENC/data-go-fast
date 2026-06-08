use crate::models::auth::RefreshTokenRow;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Saves a cryptographically hashed version of the refresh token to Postgres.
/// Returns the expiration timestamp so it can be synchronized with Redis.
pub async fn save_refresh_token(
  pool: &PgPool,
  user_id: Uuid,
  raw_token: &str,
  ttl_days: i64,
) -> Result<chrono::DateTime<Utc>, sqlx::Error> {
  let token_hash = get_hashed_token(raw_token);

  let expires_at = Utc::now() + Duration::days(ttl_days);

  sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

  Ok(expires_at)
}

pub async fn get_refresh_token(
  pool: &PgPool,
  raw_token: &str,
) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
  let token_hash = get_hashed_token(raw_token);
  let token_row = sqlx::query_as::<_, RefreshTokenRow>(
    "SELECT id, user_id, expires_at FROM refresh_tokens WHERE token_hash=$1",
  )
  .bind(token_hash)
  .fetch_optional(pool)
  .await?;

  Ok(token_row)
}

pub async fn revoke_refresh_token(pool: &PgPool, raw_token: &str) -> Result<(), sqlx::Error> {
  let token_hash: String = get_hashed_token(raw_token);
  sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
    .bind(token_hash)
    .execute(pool)
    .await?;

  Ok(())
}

fn get_hashed_token(raw_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(raw_token.as_bytes());
  hasher
    .finalize()
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect()
}
