use redis::{Client, JsonAsyncCommands, RedisResult, aio::MultiplexedConnection};
use serde_json::{Value, json};
use sqlx::{Error, Pool, Postgres, postgres::PgRow, query};
use uuid::Uuid;

pub async fn add_job_to_redis(
  pipeline: &Value,
  job_uuid: &Uuid,
  filename: &String,
  datasource_s3_id: &String,
) -> RedisResult<()> {
  let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING")
    .expect("REDIS_CONNECTION_STRING environment variable not found.");

  let client: Client = redis::Client::open(redis_connection_string)?;
  let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

  let redis_json: Value = json!({
      "name": filename,
      "status": "pending",
      "datasource_id": datasource_s3_id,
      "pipeline": pipeline
  });

  let response: () = con.json_set(job_uuid.to_string(), "$", &redis_json).await?;
  Ok(response)
}

pub async fn add_job_to_postgres(
  pool: &Pool<Postgres>,
  job_uuid: &Uuid,
  job_name: &str,
  job_actions: &Value,
) -> Result<PgRow, Error> {
  query("INSERT INTO jobs VALUES ($1, $2, $3, 'pending') RETURNING *;")
    .bind(job_uuid)
    .bind(job_name)
    .bind(job_actions)
    .fetch_one(pool)
    .await
}
