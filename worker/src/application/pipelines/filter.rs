use common::infra::database::datasource::{DatasourceType, create_datasource_from_s3};
use common::infra::database::job::{update_job_result, update_job_status};
use common::infra::s3::config::S3Instance;
use common::queue::models::{Job, Op};
use serde_json::Value;
use sqlx::{Pool, Postgres, Row, query};
use uuid::Uuid;

use crate::execute::Operation;
use crate::filter::{download_from_s3, parse_s3_id, upload_to_s3};

pub async fn filter_processing(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job, op: &Op) {
  let request = query("SELECT id, file_type FROM datasources WHERE s3_id = $1")
    .bind(&job.datasource_id)
    .fetch_one(pool)
    .await;

  let (datasource_id, file_type): (Uuid, DatasourceType) = match request {
    Ok(response) => (response.get("id"), response.get("file_type")),
    Err(e) => {
      eprintln!("Failed to get datasource file type: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  match file_type {
    DatasourceType::Csv => filter_csv(pool, s3, job, op).await,
    DatasourceType::Json => filter_json(pool, s3, job, &datasource_id, op).await,
  }
}

async fn filter_csv(pool: &Pool<Postgres>, s3: &S3Instance, job: &Job, op: &Op) {
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
    &DatasourceType::Csv,
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

async fn filter_json(
  pool: &Pool<Postgres>,
  s3: &S3Instance,
  job: &Job,
  datasource_id: &Uuid,
  op: &Op,
) {
  let (column, operator, value) = match op {
    Op::Filter {
      column,
      operator,
      value,
    } => (column, operator, value),
    _ => {
      eprintln!("Failed to extract filter data");
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

  // TO-DO: CHECKS FOR SQL INJECTION
  let sql_query = format!(
    "
    SELECT doc FROM json_table, jsonb_array_elements(document) AS doc
    WHERE datasource_id = $1 AND doc->>$2 {} $3;
  ",
    operator
  );

  println!("Query: {:?}", sql_query);

  let raw_value = match value {
    Value::String(s) => s.clone(),
    Value::Number(n) => n.to_string(),
    Value::Bool(b) => b.to_string(),
    _ => {
      eprintln!("Unsupported value type.");
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  println!(
    "datasource_id: {:?} | column: {} | raw_value: {}",
    datasource_id, column, raw_value
  );

  let combined: Vec<serde_json::Value> = match query(&sql_query)
    .bind(datasource_id)
    .bind(column)
    .bind(raw_value)
    .fetch_all(pool)
    .await
  {
    Ok(response) => {
      let mut combined: Vec<serde_json::Value> = vec![];
      for item in response {
        let json: serde_json::Value = item.get("doc");
        println!("JSON: {:?}", json);
        combined.push(json);
      }
      combined
    }
    Err(e) => {
      eprintln!("Failed to filter JSON file: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let current_bytes: &Vec<u8> = &serde_json::to_vec(&combined).unwrap();
  let new_s3_id = match upload_to_s3(s3, current_bytes, &group_uuid, "csv").await {
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
    &DatasourceType::Csv,
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
