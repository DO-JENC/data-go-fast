use common::infra::database::job::update_job_status;
use common::infra::s3::config::S3Instance;
use common::queue::models::Job;
use serde_json::Value;
use sqlx::{Pool, Postgres, Row, query};
use uuid::Uuid;

use common::infra::s3::utils::download_from_s3;

pub async fn ingest_json(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job) {
  println!("Ingestion de fichier:");
  let json_bytes = match download_from_s3(s3, &job.datasource_id).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to download from S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let json: Value = match serde_json::from_slice(&json_bytes) {
    Ok(str) => str,
    Err(e) => {
      eprintln!("Failed to convert bytes to JSON: {}", e);
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
  .bind(json)
  .fetch_one(pool)
  .await
  {
    Ok(_) => println!("JSON File ingested."),
    Err(e) => {
      eprintln!("Failed to ingest JSON file: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
    }
  }
}
