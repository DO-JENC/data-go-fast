use redis::aio::ConnectionManager;
use redis::{Client, RedisResult};

pub async fn init_redis_connection() -> RedisResult<ConnectionManager> {
  // Open the client configuration
  let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING")
    .expect("REDIS_CONNECTION_STRING environment variable not found.");
  let redis_client = Client::open(redis_connection_string)?;
  let manager = ConnectionManager::new(redis_client).await?;
  Ok(manager)
}
