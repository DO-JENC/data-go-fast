use serde_json::Value;
use sqlx::{Pool, Postgres, Row, query};
use uuid::Uuid;

use crate::queue::models::Job;

// Update a job's status (done / error) after the worker finished.
pub async fn update_job_status(
  pool: &Pool<Postgres>,
  job_id: &Uuid,
  status: &str,
) -> Result<(), sqlx::Error> {
  query("UPDATE jobs SET status = $1::job_status WHERE id = $2")
    .bind(status)
    .bind(job_id)
    .execute(pool)
    .await?;

  Ok(())
}

// Update status to 'done' AND store the result datasource id
// This is the worker's last step after a successful pipeline execution
pub async fn update_job_result(
  pool: &Pool<Postgres>,
  job_id: &Uuid,
  result_datasource_id: &Uuid,
) -> Result<(), sqlx::Error> {
  query("UPDATE jobs SET status = 'done'::job_status, result_datasource_id = $1 WHERE id = $2")
    .bind(result_datasource_id)
    .bind(job_id)
    .execute(pool)
    .await?;

  Ok(())
}

fn parse_pipeline(pipeline_json: Value) -> Result<Vec<crate::queue::models::Op>, sqlx::Error> {
  serde_json::from_value(pipeline_json).map_err(|e| sqlx::Error::Protocol(e.to_string()))
}

// Convert a PG row into a Job struct
fn row_to_job(row: &sqlx::postgres::PgRow) -> Result<Job, sqlx::Error> {
  let pipeline: Vec<crate::queue::models::Op> = parse_pipeline(row.get("pipeline"))?;
  let ds_id: String = row
    .get::<Option<String>, _>("datasource_id")
    .unwrap_or_default();

  Ok(Job {
    job_id: row.get("id"),
    datasource_id: ds_id,
    name: row.get("name"),
    pipeline,
    status: row.get::<String, _>("status"),
    result_datasource_id: row.get("result_datasource_id"),
  })
}

pub async fn get_job_by_id(
  pool: &Pool<Postgres>,
  job_id: &Uuid,
) -> Result<Option<Job>, sqlx::Error> {
  let rows = query(
    "SELECT j.id, j.name, j.pipeline, j.status::text, j.result_datasource_id,
            jd.datasource_id::text
     FROM jobs j
     LEFT JOIN job_datasources jd ON j.id = jd.job_id
     WHERE j.id = $1",
  )
  .bind(job_id)
  .fetch_all(pool)
  .await?;

  if let Some(row) = rows.into_iter().next() {
    row_to_job(&row).map(Some)
  } else {
    Ok(None)
  }
}

pub async fn list_jobs_by_group(
  pool: &Pool<Postgres>,
  group_id: &Uuid,
) -> Result<Vec<Job>, sqlx::Error> {
  let rows = query(
    "SELECT DISTINCT j.id, j.name, j.pipeline, j.status::text, j.result_datasource_id,
            jd.datasource_id::text
     FROM jobs j
     JOIN job_datasources jd ON j.id = jd.job_id
     JOIN datasources d ON jd.datasource_id = d.id
     WHERE d.group_id = $1
     ORDER BY j.id",
  )
  .bind(group_id)
  .fetch_all(pool)
  .await?;

  rows.iter().map(row_to_job).collect()
}
