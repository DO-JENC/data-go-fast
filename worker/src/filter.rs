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
use sqlx::{Pool, Postgres, Row, query};
use uuid::Uuid;

use common::infra::s3::utils::{parse_s3_id, upload_to_s3};

// Read CSV rows, keep only those matching the condition, write back as CSV
pub fn apply_filter(
  content: &[u8],
  column: &str,
  operator: &str,
  target: &Value,
) -> Result<Vec<u8>, String> {
  let mut reader = Reader::from_reader(content);
  let headers = reader
    .headers()
    .map_err(|e| format!("Failed to read CSV headers: {}", e))?
    .clone();

  // Find which column index to compare against
  let col_index = headers
    .iter()
    .position(|h| h.eq_ignore_ascii_case(column))
    .ok_or_else(|| format!("Column '{}' not found in CSV", column))?;

  let mut result = Vec::new();
  {
    let mut writer = Writer::from_writer(&mut result);

    // Always keep the header row
    writer
      .write_record(&headers)
      .map_err(|e| format!("Failed to write CSV headers: {}", e))?;

    // Iterate through all data rows
    for record in reader.records() {
      let record = record.map_err(|e| format!("Failed to read CSV record: {}", e))?;
      let cell = record.get(col_index).unwrap_or("");

      // Keep the row only if the cell satisfies the condition
      if evaluate(cell, operator, target)? {
        writer
          .write_record(&record)
          .map_err(|e| format!("Failed to write CSV record: {}", e))?;
      }
    }
  }

  Ok(result)
}

// Check if a cell value matches the given condition
// Order: try numeric → mixed → string
fn evaluate(cell: &str, operator: &str, target: &Value) -> Result<bool, String> {
  // Both are numbers → compare numerically (supports all operators)
  if let (Ok(cell_num), Some(target_num)) = (cell.parse::<f64>(), target.as_f64()) {
    return match operator {
      ">" => Ok(cell_num > target_num),
      "<" => Ok(cell_num < target_num),
      ">=" => Ok(cell_num >= target_num),
      "<=" => Ok(cell_num <= target_num),
      "==" => Ok((cell_num - target_num).abs() < f64::EPSILON),
      "!=" => Ok((cell_num - target_num).abs() >= f64::EPSILON),
      _ => Err(format!("Unsupported operator: {}", operator)),
    };
  }

  // One is a number, the other is not
  //    == is always false, != is always true
  //    comparison operators (>, <, ...) return an error
  if cell.parse::<f64>().is_ok() && target.as_f64().is_none() {
    return match operator {
      "==" => Ok(false),
      "!=" => Ok(true),
      _ => Err(format!(
        "Operator '{}' requires numeric values on both sides",
        operator
      )),
    };
  }

  if cell.parse::<f64>().is_err() && target.as_f64().is_some() {
    return match operator {
      "==" => Ok(false),
      "!=" => Ok(true),
      _ => Err(format!(
        "Operator '{}' requires numeric values on both sides",
        operator
      )),
    };
  }

  // Both are strings → only == and != are allowed (case-insensitive)
  let target_owned = target.to_string();
  let target_str = target.as_str().unwrap_or(&target_owned);
  match operator {
    "==" => Ok(cell.to_lowercase() == target_str.to_lowercase()),
    "!=" => Ok(cell.to_lowercase() != target_str.to_lowercase()),
    _ => Err(format!(
      "Operator '{}' is not supported for string values",
      operator
    )),
  }
}

pub async fn filter_json(
  pool: &Pool<Postgres>,
  s3: &S3Instance,
  job: &Job,
  datasource_id: &Uuid,
  op: &Op,
) {
  // Extract column, operator and value from Operation
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

  // Prevent SQL injection
  let normalized_operator = match operator.as_str() {
    "=" => "=",
    "==" => "=",
    "!=" => "!=",
    "<" => "<",
    ">" => ">",
    "<=" => "<=",
    ">=" => ">=",
    other => {
      eprintln!("Rejected unsafe operator: {}", other);
      let _ = update_job_status(pool, &job.job_id, "error").await;
      return;
    }
  };

  let sql_query = format!(
    "
    SELECT doc FROM json_table, jsonb_array_elements(document) AS doc
    WHERE datasource_id = $1 AND doc->>$2 {} $3;
  ",
    normalized_operator
  );

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

  let current_bytes: Vec<u8> = match serde_json::to_vec(&combined) {
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
