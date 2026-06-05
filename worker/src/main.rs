mod execute;
mod filter;

mod application;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use common::infra::database::config::create_pool_from_env;
use common::infra::database::datasource::{DatasourceType, create_datasource_from_s3};
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::init_s3_instance;
use common::queue::models::{Job, Op};
// use application::pipelines::ingest::ingest_json;
use common::infra::database::datasource::DatasourceType::{Csv, Json};
use common::infra::s3::config::S3Instance;
use common::queue::storage::get_queue_storage;
use sqlx::{Pool, Postgres};

use crate::execute::Operation;
use crate::filter::{download_from_s3, parse_s3_id, upload_to_s3};

async fn job_treatment(job: Job) {
  println!("Processing job: {:?}", job.job_id);

  // Init S3 Instance and Postgres Pool
  let s3 = init_s3_instance();
  let pool = match create_pool_from_env().await {
    Ok(p) => p,
    Err(e) => {
      eprintln!("Failed to create database pool: {}", e);
      return;
    }
  };

  // Update Job status to running
  let _ = update_job_status(&pool, &job.job_id, "running").await;

  // Checks which operation to process
  for op in &job.pipeline {
    match op {
      Op::Ingest { r#type, .. } => match r#type {
        DatasourceType::Csv => println!("CSV Ingestion has been process."),
        DatasourceType::Json => println!("JSON Ingestion has been process."),
      },
      Op::Filter { .. }=> filter_processing(&pool, &s3, &job, &op).await
    }
  }
}

async fn filter_processing(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job, op: &Op ) {
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

  match create_datasource_from_s3(pool, &new_s3_id, &base_name, &group_uuid, size_mb).await {
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
