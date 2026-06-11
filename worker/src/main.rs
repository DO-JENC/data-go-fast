mod aggregate;
mod execute;
mod filter;
mod group_by;
mod ingest;
mod utils;

use apalis::prelude::*;
use apalis_redis::RedisStorage;
use common::infra::database::config::create_pool_from_env;
use common::infra::database::datasource::{DatasourceType, create_datasource_from_s3};
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::{S3Instance, init_s3_instance};
use common::logs::init_logging;
use common::queue::models::{Job, Op};
use common::queue::storage::get_queue_storage;
use sqlx::{Pool, Postgres, Row, query};
use tracing::{error, info, instrument};

use crate::execute::Operation;
use crate::ingest::ingest_json;
use crate::utils::{download_from_s3, parse_s3_id, upload_to_s3};

#[instrument(skip_all, fields(job_id = %job.job_id))]
async fn job_processing(
  job: Job,
  pool_data: apalis::prelude::Data<
    common::infra::database::Pool<common::infra::database::Postgres>,
  >,
  s3_data: apalis::prelude::Data<common::infra::s3::config::S3Instance>,
) {
  info!("Processing job");

  let pool = &*pool_data;
  let s3 = &*s3_data;

  let request = query("SELECT file_type FROM datasources WHERE s3_id = $1")
    .bind(&job.datasource_id)
    .fetch_one(pool)
    .await;

  let file_type: DatasourceType = match request {
    Ok(response) => response.get("file_type"),
    Err(e) => {
      eprintln!("Failed to get datasource file type: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let _ = update_job_status(pool, &job.job_id, "running").await;

  match file_type {
    DatasourceType::Csv => csv_processing(job, s3, pool).await,
    DatasourceType::Json => json_processing(job, s3, pool).await,
  }
}

async fn csv_processing(job: Job, s3: &S3Instance, pool: &Pool<Postgres>) {
  let csv_bytes = match download_from_s3(s3, &job.datasource_id).await {
    Ok(bytes) => bytes,
    Err(e) => {
      error!("Failed to download from S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let (group_uuid, _, _) = match parse_s3_id(&job.datasource_id) {
    Ok(t) => t,
    Err(e) => {
      error!("Failed to parse S3 ID: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
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
        error!("Operation failed for job {}: {}", job.job_id, e);
        let _ = update_job_status(pool, &job.job_id, "error").await;
        return;
      }
    }
  }

  let is_aggregate = job
    .pipeline
    .iter()
    .any(|op| matches!(op, Op::Aggregate { .. }));
  let ext = if is_aggregate { "json" } else { "csv" };

  let new_s3_id = match upload_to_s3(s3, &current_bytes, &group_uuid, ext).await {
    Ok(id) => id,
    Err(e) => {
      error!("Failed to upload to S3: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
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
    pool,
    &new_s3_id,
    &base_name,
    &group_uuid,
    size_mb,
    file_type,
  )
  .await
  {
    Ok(new_id) => {
      info!("Datasource created with ID: {}", new_id);
      if let Err(e) = update_job_result(pool, &job.job_id, &new_id).await {
        error!("Failed to update job result: {}", e);
      }
    }
    Err(e) => {
      error!("Failed to create datasource: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
    }
  }
}

async fn shutdown_signal() {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install SIGTERM handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}

async fn json_processing(job: Job, s3: &S3Instance, pool: &Pool<Postgres>) {
  println!("JSON Processing");
  for op in &job.pipeline {
    match op {
      Op::Ingest { .. } => {
        ingest_json(pool, s3, &job).await;
        return;
      }
      Op::Filter { .. } => println!("JSON Filtering not yet implemented"),
      Op::Aggregate { .. } => println!("JSON Aggregating not yet implemented"),
      Op::GroupBy { .. } => println!("JSON GroupBying not yet implemented"),
    }
  }
}

#[tokio::main]
async fn main() {
  init_logging();
  info!("Starting worker...");

  // Initialize once on startup
  let pool = create_pool_from_env()
    .await
    .expect("Failed to create DB pool");
  let s3 = init_s3_instance();
  let storage: RedisStorage<Job> = get_queue_storage().await;

  let monitor = Monitor::new().register(
    WorkerBuilder::new("worker")
      .concurrency(2)
      .data(pool)
      .data(s3)
      .backend(storage)
      .build_fn(job_processing),
  );

  tokio::select! {
    _ = monitor.run() => {},
    _ = shutdown_signal() => {
      println!("Shutdown signal received, waiting for in-flight jobs to complete...");
    },
  }
}
