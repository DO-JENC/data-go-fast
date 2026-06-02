use crate::queue::models::Job;
use apalis_redis::{Config, ConnectionManager, RedisStorage};

pub async fn get_queue_storage() -> RedisStorage<Job, ConnectionManager> {
  let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING")
    .expect("REDIS_CONNECTION_STRING environment variable not found.");

  let conn = apalis_redis::connect(redis_connection_string)
    .await
    .expect("Could not connect");
  let config = Config::default().set_namespace("data-go-fast");

  RedisStorage::new_with_config(conn, config)
}
