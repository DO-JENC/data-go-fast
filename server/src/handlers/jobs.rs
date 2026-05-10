use redis::{Client, JsonAsyncCommands, RedisResult, aio::MultiplexedConnection};
use serde_json::{Value, json};
use sqlx::{Error, Pool, Postgres, postgres::PgRow, query};
use uuid::Uuid;

pub async fn add_job_to_redis(
  pipeline: &Value,
  job_uuid: &Uuid,
  job_name: &str,
  datasource_s3_id: &str,
) -> RedisResult<()> {
  let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING")
    .expect("REDIS_CONNECTION_STRING environment variable not found.");

  let client: Client = redis::Client::open(redis_connection_string)?;
  let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

  let redis_json: Value = json!({
      "name": job_name,
      "status": "pending",
      "datasource_id": datasource_s3_id,
      "pipeline": pipeline
  });

  let response: () = con.json_set(job_uuid.to_string(), "$", &redis_json).await?;
  Ok(response)
}

pub async fn add_job_to_postgres(
  pool: &Pool<Postgres>,
  pipeline: &Value,
  job_uuid: &Uuid,
  job_name: &str,
) -> Result<PgRow, Error> {
  query("INSERT INTO jobs VALUES ($1, $2, $3, 'pending') RETURNING *;")
    .bind(job_uuid)
    .bind(job_name)
    .bind(pipeline)
    .fetch_one(pool)
    .await
}
