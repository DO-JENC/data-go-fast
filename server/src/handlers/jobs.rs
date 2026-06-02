use crate::AppState;
use apalis::prelude::*;
use apalis_core::request::Parts;
use apalis_redis::{RedisContext, RedisStorage};
use common::queue::models::{Job, Pipeline};
use sqlx::{Error, Pool, Postgres, postgres::PgRow, query, types::Json};
use uuid::Uuid;

pub async fn add_job_to_redis(
  redis_conn: &RedisStorage<Job>,
  pipeline: &Pipeline,
  job_uuid: &Uuid,
  job_name: &str,
  datasource_s3_id: &str,
) -> Result<Parts<RedisContext>, Error> {
  let response: Parts<RedisContext> = redis_conn
    .clone()
    .push(Job {
      job_id: *job_uuid,
      datasource_id: datasource_s3_id.into(),
      name: job_name.into(),
      pipeline: pipeline.clone(),
      status: "pending".into(),
    })
    .await
    .expect("Failed to push job");

  Ok(response)
}

pub async fn add_job_to_postgres(
  pool: &Pool<Postgres>,
  pipeline: &Pipeline,
  job_uuid: &Uuid,
  job_name: &str,
  file_uuid: &Uuid,
) -> Result<PgRow, Error> {
  let job_row = query("INSERT INTO jobs VALUES ($1, $2, $3, 'pending') RETURNING *;")
    .bind(job_uuid)
    .bind(job_name)
    .bind(Json(pipeline))
    .fetch_one(pool)
    .await?;

  query("INSERT INTO job_datasources VALUES ($1, $2) RETURNING *;")
    .bind(job_uuid)
    .bind(file_uuid)
    .fetch_one(pool)
    .await?;

  Ok(job_row)
}
