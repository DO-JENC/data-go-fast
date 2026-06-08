mod execute;
mod filter;

mod application;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use application::pipelines::filter::filter_processing;
use application::pipelines::ingest::ingest_json;
use common::infra::database::config::create_pool_from_env;
use common::infra::database::datasource::DatasourceType;
use common::infra::database::job::update_job_status;
use common::infra::s3::config::init_s3_instance;
use common::queue::models::{Job, Op};
use common::queue::storage::get_queue_storage;

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
        &DatasourceType::Csv => println!("CSV Ingestion has been process."),
        &DatasourceType::Json => ingest_json(&pool, &s3, &job).await,
      },
      Op::Filter { .. } => filter_processing(&pool, &s3, &job, &op).await,
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
