use redis::aio::ConnectionManager;
use redis::{Client, RedisResult};
use tracing::{info, instrument};

#[instrument]
pub async fn init_redis_connection() -> RedisResult<ConnectionManager> {
  // Open the client configuration
  let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING")
    .expect("REDIS_CONNECTION_STRING environment variable not found.");

  let masked_url = if let Ok(url) = url::Url::parse(&redis_connection_string) {
    let mut masked = url.clone();
    if masked.password().is_some() {
      let _ = masked.set_password(Some("********"));
    }
    masked.to_string()
  } else {
    "invalid-url".to_string()
  };

  info!("Initializing Redis connection manager to {}", masked_url);

  let redis_client = Client::open(redis_connection_string)?;
  let manager = ConnectionManager::new(redis_client).await?;

  info!("Redis connection manager initialized");
  Ok(manager)
}
