use common::{
  infra::{
    database::{
      datasource::{DatasourceType, create_datasource_from_s3},
      job::{update_job_result, update_job_status},
    },
    s3::config::S3Instance,
  },
  queue::models::{Job, Op},
};
use csv::Reader;
use serde_json::{Map, Value};
use sqlx::{Pool, Postgres, Row};
use std::collections::HashMap;
use uuid::Uuid;

use common::infra::s3::utils::{parse_s3_id, upload_to_s3};

pub fn sum(values: &[f64]) -> f64 {
  values.iter().sum()
}

pub fn avg(values: &[f64]) -> f64 {
  sum(values) / values.len() as f64
}

pub fn min(values: &[f64]) -> f64 {
  values
    .iter()
    .copied()
    .reduce(f64::min)
    .unwrap_or(f64::INFINITY)
}

pub fn max(values: &[f64]) -> f64 {
  values
    .iter()
    .copied()
    .reduce(f64::max)
    .unwrap_or(f64::NEG_INFINITY)
}

pub fn count(values: &[f64]) -> f64 {
  values.len() as f64
}

pub fn median(values: &[f64]) -> f64 {
  let mut sorted = values.to_vec();
  sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
  let mid = sorted.len() / 2;
  if sorted.len().is_multiple_of(2) {
    (sorted[mid - 1] + sorted[mid]) / 2.0
  } else {
    sorted[mid]
  }
}

fn compute(values: &[f64], func: &str) -> Result<Value, String> {
  match func {
    "sum" => Ok(Value::from(sum(values))),
    "avg" => Ok(Value::from(avg(values))),
    "min" => Ok(Value::from(min(values))),
    "max" => Ok(Value::from(max(values))),
    "count" => Ok(Value::from(count(values))),
    "median" => Ok(Value::from(median(values))),
    _ => Err(format!("Unknown function: '{}'", func)),
  }
}

pub fn aggregate_csv(
  content: &[u8],
  columns: &[String],
  functions: &[String],
) -> Result<Vec<u8>, String> {
  let mut reader = Reader::from_reader(content);
  let headers = reader
    .headers()
    .map_err(|e| format!("Failed to read CSV headers: {}", e))?
    .clone();

  // Resolve each column name to its index in the CSV header
  let col_indices: Vec<(String, usize)> = columns
    .iter()
    .map(|col| {
      let idx = headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(col))
        .ok_or_else(|| format!("Column '{}' not found in CSV", col))?;
      Ok((col.clone(), idx))
    })
    .collect::<Result<Vec<_>, String>>()?;

  // Extract numeric values: for each record, parse every requested cell
  let mut col_values: HashMap<String, Vec<f64>> = HashMap::new();
  for record in reader.records() {
    let record = record.map_err(|e| format!("Failed to read CSV record: {}", e))?;
    for (col_name, idx) in &col_indices {
      let cell = record.get(*idx).unwrap_or("");
      let val: f64 = cell
        .parse()
        .map_err(|_| format!("Value '{}' in column '{}' is not a number", cell, col_name))?;
      col_values.entry(col_name.clone()).or_default().push(val);
    }
  }

  // Build JSON: { "col": { "func": result, ... }, ... }
  let mut root = Map::new();
  for (col_name, _) in &col_indices {
    let values = col_values.remove(col_name).unwrap_or_default();
    if values.is_empty() {
      return Err(format!("Column '{}' has no numeric values", col_name));
    }
    let mut func_map = Map::new();
    for func in functions {
      func_map.insert(func.clone(), compute(&values, func)?);
    }
    root.insert(col_name.clone(), Value::Object(func_map));
  }

  serde_json::to_vec(&Value::Object(root)).map_err(|e| format!("Failed to serialize JSON: {}", e))
}

pub async fn json_aggregate(
  pool: &Pool<Postgres>,
  s3: &S3Instance,
  job: &Job,
  datasource_id: &Uuid,
  op: &Op,
) {
  // Extract columns and functions from Operation
  let (columns, functions) = match op {
    Op::Aggregate { columns, functions } => (columns, functions),
    _ => {
      eprintln!("Failed to extract aggregate arguments");
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

  // Build SELECT clause: SUM((doc->>'col')::numeric), AVG((doc->>'col')::numeric), ...
  let select_parts: Vec<String> = columns
    .iter()
    .flat_map(|col| {
      functions.iter().map(move |func| {
        let func_lower = func.to_lowercase(); // bind to a local variable
        let sql_func = match func_lower.as_str() {
          "avg" => "AVG",
          "sum" => "SUM",
          "min" => "MIN",
          "max" => "MAX",
          "count" => "COUNT",
          other => other,
        };
        format!(
          "{}((doc->>'{}')::numeric)::float8 AS \"{}_{}\"",
          sql_func, col, col, func_lower
        )
      })
    })
    .collect();

  let query = format!(
    "SELECT {} FROM json_table, jsonb_array_elements(document) AS doc WHERE datasource_id = $1",
    select_parts.join(", ")
  );

  let row = match sqlx::query(&query)
    .bind(datasource_id)
    .fetch_one(pool)
    .await
  {
    Ok(row) => row,
    Err(e) => {
      eprintln!("Aggregate query failed: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  // Build result: { "Rating": { "avg": 3.41, "sum": 1126.0 }, "Year": { ... } }
  let mut result: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

  for col in columns {
    let mut col_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for func in functions {
      let label = format!("{}_{}", col, func);
      let value: Option<f64> = row.try_get(label.as_str()).unwrap_or(None);
      col_map.insert(
        func.to_lowercase(),
        match value {
          Some(v) => serde_json::Value::from(v),
          None => serde_json::Value::Null,
        },
      );
    }

    result.insert(col.clone(), serde_json::Value::Object(col_map));
  }

  let json_result = serde_json::Value::Object(result);

  let current_bytes: Vec<u8> = match serde_json::to_vec(&json_result) {
    Ok(current_bytes) => current_bytes,
    Err(e) => {
      eprintln!("Failed to convert JSON to Vec<u8>: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let new_s3_id = match upload_to_s3(s3, &current_bytes, &group_uuid, "json").await {
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
    &group_uuid,
    size_mb,
    DatasourceType::Json,
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
