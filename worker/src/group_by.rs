use common::infra::s3::utils::{parse_s3_id, upload_to_s3};
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
use csv::{Reader, Writer};
use serde_json::Value;
use sqlx::{Pool, Postgres, Row};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::aggregate;

fn find_indices(
  headers: &csv::StringRecord,
  by: &str,
  column: &str,
) -> Result<(usize, usize), String> {
  let by_idx = headers
    .iter()
    .position(|h| h.eq_ignore_ascii_case(by))
    .ok_or_else(|| format!("Column '{}' not found in CSV", by))?;

  let col_idx = headers
    .iter()
    .position(|h| h.eq_ignore_ascii_case(column))
    .ok_or_else(|| format!("Column '{}' not found in CSV", column))?;

  Ok((by_idx, col_idx))
}

fn group_values(
  reader: &mut Reader<&[u8]>,
  by_idx: usize,
  col_idx: usize,
  column: &str,
) -> Result<BTreeMap<String, Vec<f64>>, String> {
  let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
  for record in reader.records() {
    let record = record.map_err(|e| format!("Failed to read CSV record: {}", e))?;
    let key = record.get(by_idx).unwrap_or("").to_string();
    let cell = record.get(col_idx).unwrap_or("");
    let val: f64 = cell
      .parse()
      .map_err(|_| format!("Value '{}' in column '{}' is not a number", cell, column))?;
    groups.entry(key).or_default().push(val);
  }
  Ok(groups)
}

pub(crate) fn compute_f64(values: &[f64], func: &str) -> Result<f64, String> {
  match func {
    "sum" => Ok(aggregate::sum(values)),
    "avg" => Ok(aggregate::avg(values)),
    "min" => Ok(aggregate::min(values)),
    "max" => Ok(aggregate::max(values)),
    "count" => Ok(aggregate::count(values)),
    "median" => Ok(aggregate::median(values)),
    _ => Err(format!("Unknown function: '{}'", func)),
  }
}

fn write_csv(
  groups: &BTreeMap<String, Vec<f64>>,
  by: &str,
  column: &str,
  function: &str,
) -> Result<Vec<u8>, String> {
  let mut result = Vec::new();
  {
    let mut writer = Writer::from_writer(&mut result);
    writer
      .write_record([by, &format!("{}_{}", column, function)])
      .map_err(|e| format!("Failed to write CSV header: {}", e))?;

    for (key, values) in groups {
      if values.is_empty() {
        return Err(format!("Group '{}' has no numeric values", key));
      }
      let val = compute_f64(values, function)?;
      writer
        .write_record([key, &val.to_string()])
        .map_err(|e| format!("Failed to write CSV record: {}", e))?;
    }
  }
  Ok(result)
}

pub fn group_by_csv(
  content: &[u8],
  by: &str,
  column: &str,
  function: &str,
) -> Result<Vec<u8>, String> {
  let mut reader = Reader::from_reader(content);
  let headers = reader
    .headers()
    .map_err(|e| format!("Failed to read CSV headers: {}", e))?
    .clone();

  let (by_idx, col_idx) = find_indices(&headers, by, column)?;
  let groups = group_values(&mut reader, by_idx, col_idx, column)?;
  write_csv(&groups, by, column, function)
}

pub async fn json_group_by(
  pool: &Pool<Postgres>,
  s3: &S3Instance,
  job: &Job,
  datasource_id: &Uuid,
  op: &Op,
) {
  let (by, column, function) = match op {
    Op::GroupBy { by, aggregate } => (by, &aggregate.column, &aggregate.function),
    _ => {
      eprintln!("Failed to extract group_by arguments");
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

  let func_lower = function.to_lowercase();
  let sql_func = match func_lower.as_str() {
    "avg" => "AVG",
    "sum" => "SUM",
    "min" => "MIN",
    "max" => "MAX",
    "count" => "COUNT",
    other => other,
  };

  let result_label = format!("{}_{}", column, func_lower);

  let query = format!(
    "SELECT doc->>'{}' AS group_key, {}((doc->>'{}')::numeric)::float8 AS \"{}\" \
     FROM json_table, jsonb_array_elements(document) AS doc \
     WHERE datasource_id = $1 \
     GROUP BY doc->>'{}' \
     ORDER BY group_key",
    by, sql_func, column, result_label, by
  );

  println!("QUERY: {:?}", query);

  let rows = match sqlx::query(&query)
    .bind(datasource_id)
    .fetch_all(pool)
    .await
  {
    Ok(rows) => rows,
    Err(e) => {
      eprintln!("GroupBy query failed: {}", e);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let mut result: Vec<Value> = Vec::new();

  for row in rows {
    let group_key: String = row.try_get("group_key").unwrap_or_default();
    let agg_value: Option<f64> = row.try_get(result_label.as_str()).unwrap_or(None);

    let mut map = serde_json::Map::new();

    let key_value: Value = match group_key.parse::<f64>() {
      Ok(n) => Value::from(n),
      Err(_) => Value::String(group_key),
    };

    map.insert(by.clone(), key_value);
    map.insert(
      result_label.clone(),
      match agg_value {
        Some(v) => Value::from(v),
        None => Value::Null,
      },
    );

    result.push(Value::Object(map));
  }

  let json_result = Value::Array(result);
  println!("JSON RESULT: {:?}", json_result);

  let current_bytes: Vec<u8> = match serde_json::to_vec(&json_result) {
    Ok(bytes) => bytes,
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
