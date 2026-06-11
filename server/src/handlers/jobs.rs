use apalis::prelude::*;
use apalis_core::request::Parts;
use apalis_redis::{RedisContext, RedisStorage};
use axum::{
  Json,
  extract::{Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use common::queue::models::{Job, Pipeline};
use serde::Deserialize;
use sqlx::{Pool, Postgres, Row, query, types::Json as JsonSqlx};
use uuid::Uuid;

use crate::AppState;

pub async fn add_job_to_redis(
  redis_conn: &RedisStorage<Job>,
  pipeline: &Pipeline,
  job_uuid: &Uuid,
  job_name: &str,
  datasource_s3_id: &str,
) -> Result<Parts<RedisContext>, sqlx::Error> {
  let response: Parts<RedisContext> = redis_conn
    .clone()
    .push(Job {
      job_id: *job_uuid,
      datasource_id: datasource_s3_id.into(),
      name: job_name.into(),
      pipeline: pipeline.clone(),
      status: "pending".into(),
      result_datasource_id: None,
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
) -> Result<(), sqlx::Error> {
  query("INSERT INTO jobs (id, name, pipeline, status) VALUES ($1, $2, $3, 'pending')")
    .bind(job_uuid)
    .bind(job_name)
    .bind(JsonSqlx(pipeline))
    .execute(pool)
    .await?;

  query("INSERT INTO job_datasources (job_id, datasource_id) VALUES ($1, $2)")
    .bind(job_uuid)
    .bind(file_uuid)
    .execute(pool)
    .await?;

  Ok(())
}

#[derive(Deserialize)]
pub struct CreateJobRequest {
  pub name: String,
  pub datasource_id: Uuid,
  pub pipeline: Pipeline,
}

pub async fn create_job_handler(
  State(state): State<AppState>,
  Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
  // Validate the datasource exists and get its S3 path
  let s3_id: String = match query("SELECT s3_id FROM datasources WHERE id = $1")
    .bind(req.datasource_id)
    .fetch_optional(&state.pool)
    .await
  {
    Ok(Some(row)) => row.get("s3_id"),
    Ok(None) => return (StatusCode::NOT_FOUND, "Datasource not found".to_string()),
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
  };

  let job_uuid = Uuid::new_v4();
  let job_name = req.name;

  // Push to Redis
  if let Err(e) =
    add_job_to_redis(&state.storage, &req.pipeline, &job_uuid, &job_name, &s3_id).await
  {
    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
  }

  // Persist in Postgres
  if let Err(e) = add_job_to_postgres(
    &state.pool,
    &req.pipeline,
    &job_uuid,
    &job_name,
    &req.datasource_id,
  )
  .await
  {
    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
  }

  (
    StatusCode::ACCEPTED,
    serde_json::json!({ "job_id": job_uuid }).to_string(),
  )
}

pub async fn get_job_by_id_handler(
  State(state): State<AppState>,
  Path(job_id): Path<Uuid>,
) -> Result<Json<Job>, (StatusCode, String)> {
  let job = common::infra::database::job::get_job_by_id(&state.pool, &job_id)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Job not found".to_string()))?;

  Ok(Json(job))
}

#[derive(Deserialize)]
pub struct ListJobsQuery {
  pub group_id: Option<Uuid>,
}

pub async fn list_jobs_handler(
  State(state): State<AppState>,
  Query(query): Query<ListJobsQuery>,
) -> Result<Json<Vec<Job>>, (StatusCode, String)> {
  let group_id = match query.group_id {
    Some(id) => id,
    None => return Ok(Json(vec![])),
  };

  let jobs = common::infra::database::job::list_jobs_by_group(&state.pool, &group_id)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  Ok(Json(jobs))
}
