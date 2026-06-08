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
  let mut hasher = Sha256::new();
  hasher.update(raw_token.as_bytes());
  let token_hash: String = hasher
    .finalize()
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect();

  let expires_at = Utc::now() + Duration::days(ttl_days);

  sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

  Ok(expires_at)
}

pub async fn revoke_refresh_token(pool: &PgPool, raw_token: &str) -> Result<(), sqlx::Error> {
  let mut hasher = Sha256::new();
  hasher.update(raw_token.as_bytes());
  let token_hash: String = hasher
    .finalize()
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect();
  sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
    .bind(token_hash)
    .execute(pool)
    .await?;

  Ok(())
}
