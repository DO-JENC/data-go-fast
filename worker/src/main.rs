mod execute;
mod filter;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use common::infra::database::config::create_pool_from_env;
use common::infra::database::datasource::{create_datasource_from_s3, get_unique_datasource_name};
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::init_s3_instance;
use common::queue::models::{Job, Op};
use common::queue::storage::get_queue_storage;

use crate::execute::Operation;

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

  // Iterate over each operation in the pipeline sequentially
  for op in &job.pipeline {
    // Server handles Ingest synchronously at upload time; skip in worker
    if matches!(op, Op::Ingest { .. }) {
      continue;
    }
    match op.execute(&job, &s3).await {
      Ok((new_s3_id, original_name, group_id, size_mb)) => {
        // Generate a unique name (e.g. "sales_filtered", "sales_filtered(1)")
        let new_name =
          get_unique_datasource_name(&pool, &format!("{}_filtered", original_name), &group_id)
            .await
            .unwrap_or(format!("{}_filtered", original_name));

        // Persist the new datasource in Postgres
        match create_datasource_from_s3(&pool, &new_s3_id, &new_name, &group_id, size_mb).await {
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
      Err(e) => {
        eprintln!("Operation failed for job {}: {}", job.job_id, e);
        let _ = update_job_status(&pool, &job.job_id, "error").await;
      }
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
