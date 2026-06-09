mod aggregate;
mod execute;
mod filter;
mod group_by;
mod utils;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use common::infra::database::config::create_pool_from_env;
use common::infra::database::datasource::create_datasource_from_s3;
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::init_s3_instance;
use common::queue::models::{Job, Op};
use common::queue::storage::get_queue_storage;

use crate::execute::Operation;
use crate::utils::{download_from_s3, parse_s3_id, upload_to_s3};

async fn job_treatment(job: Job) {
  println!("Processing job: {:?}", job.job_id);

  let s3 = init_s3_instance();
  let pool = match create_pool_from_env().await {
    Ok(p) => p,
    Err(e) => {
      eprintln!("Failed to create database pool: {}", e);
      return;
    }
  };

  let _ = update_job_status(&pool, &job.job_id, "running").await;

  let csv_bytes = match download_from_s3(&s3, &job.datasource_id).await {
    Ok(bytes) => bytes,
    Err(e) => {
      eprintln!("Failed to download from S3: {}", e);
      let _ = update_job_status(&pool, &job.job_id, "error").await;
      return;
    }
  };

  let (group_uuid, _, _) = match parse_s3_id(&job.datasource_id) {
    Ok(t) => t,
    Err(e) => {
      eprintln!("Failed to parse S3 ID: {}", e);
      let _ = update_job_status(&pool, &job.job_id, "error").await;
      return;
    }
  };

  let mut current_bytes = csv_bytes;
  for op in &job.pipeline {
    if matches!(op, Op::Ingest { .. }) {
      continue;
    }
    match op.execute_on_bytes(&current_bytes).await {
      Ok(filtered) => current_bytes = filtered,
      Err(e) => {
        eprintln!("Operation failed for job {}: {}", job.job_id, e);
        let _ = update_job_status(&pool, &job.job_id, "error").await;
        return;
      }
    }
  }

  let is_aggregate = job
    .pipeline
    .iter()
    .any(|op| matches!(op, Op::Aggregate { .. }));
  let ext = if is_aggregate { "json" } else { "csv" };

  let new_s3_id = match upload_to_s3(&s3, &current_bytes, &group_uuid, ext).await {
    Ok(id) => id,
    Err(e) => {
      eprintln!("Failed to upload to S3: {}", e);
      let _ = update_job_status(&pool, &job.job_id, "error").await;
      return;
    }
  };

  let size_mb = current_bytes.len() as f64 / (1024.0 * 1024.0);
  let base_name = job.name.clone();

  let file_type = if is_aggregate {
    common::infra::database::datasource::DatasourceType::Json
  } else {
    common::infra::database::datasource::DatasourceType::Csv
  };

  match create_datasource_from_s3(
    &pool,
    &new_s3_id,
    &base_name,
    &group_uuid,
    size_mb,
    file_type,
  )
  .await
  {
    Ok(new_id) => {
      println!("Datasource created with ID: {}", new_id);
      if let Err(e) = update_job_result(&pool, &job.job_id, &new_id).await {
        eprintln!("Failed to update job result: {}", e);
      }
    }
    Err(e) => {
      eprintln!("Failed to create datasource: {}", e);
      let _ = update_job_status(&pool, &job.job_id, "error").await;
    }
  }
}

#[tokio::main]
async fn main() {
  let storage: RedisStorage<Job> = get_queue_storage().await;
  Monitor::new()
    .register(
      WorkerBuilder::new("worker")
        .concurrency(2)
        .backend(storage)
        .build_fn(job_treatment),
    )
    .run()
    .await
    .expect("Monitor failed");
}
