use common::infra::database::datasource::{DatasourceType, create_datasource_from_s3};
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::S3Instance;
use common::queue::models::{Job, Op};
use sqlx::{Pool, Postgres};

use crate::execute::Operation;
use crate::filter::{download_from_s3, parse_s3_id, upload_to_s3};

pub async fn filter_processing(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job, op: &Op) {
  let csv_bytes = match download_from_s3(s3, &job.datasource_id).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to download from S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let (group_uuid, _, _) = match parse_s3_id(&job.datasource_id) {
    Ok(t) => t,
    Err(e) => {
      eprintln!("Failed to parse S3 ID: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let mut current_bytes = csv_bytes;
  match op.execute_on_bytes(&current_bytes).await {
    Ok(filtered) => current_bytes = filtered,
    Err(e) => {
      eprintln!("Operation failed for job {}: {}", job.job_id, e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  }

  let new_s3_id = match upload_to_s3(s3, &current_bytes, &group_uuid, "csv").await {
    Ok(id) => id,
    Err(e) => {
      eprintln!("Failed to upload to S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let size_mb = current_bytes.len() as f64 / (1024.0 * 1024.0);
  let base_name = job.name.clone();

  match create_datasource_from_s3(
    pool,
    &new_s3_id,
    &base_name,
    &DatasourceType::Json,
    &group_uuid,
    size_mb,
  )
  .await
  {
    Ok(new_id) => {
      println!("Datasource created with ID: {}", new_id);
      if let Err(e) = update_job_result(pool, &job.job_id, &new_id).await {
        eprintln!("Failed to update job result: {}", e);
      }
    }
    Err(e) => {
      eprintln!("Failed to create datasource: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
    }
  }
}
