use common::infra::database::job::update_job_status;
use common::infra::s3::config::S3Instance;
use common::queue::models::Job;
use sqlx::Row;
use sqlx::types::Json;
use sqlx::{Pool, Postgres, query};
use uuid::Uuid;

use crate::filter::download_from_s3;

pub async fn ingest_json(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job) {
  let json_bytes = match download_from_s3(s3, &job.datasource_id).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to download from S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let json_string: String = match String::from_utf8(json_bytes) {
    Ok(str) => str,
    Err(e) => {
      eprintln!("Failed to convert bytes to string: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let request = query("SELECT id FROM datasources WHERE s3_id = $1")
    .bind(&job.datasource_id)
    .fetch_one(pool)
    .await;

  let datasource_id: Uuid = match request {
    Ok(response) => response.get("id"),
    Err(e) => {
      eprintln!("Failed to get datasource ID: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  match query(
    "
            INSERT INTO
            json_table (datasource_id, document)
            VALUES ($1, $2) RETURNING *;
        ",
  )
  .bind(datasource_id)
  .bind(Json(json_string))
  .fetch_one(pool)
  .await
  {
    Ok(_) => println!("JSON File ingested."),
    Err(e) => {
      eprintln!("Failed to ingest JSON file: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  }
}
