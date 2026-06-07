use common::infra::s3::config::S3Instance;
use csv::{Reader, Writer};
use s3::Bucket;
use serde_json::Value;
use uuid::Uuid;

// Download raw bytes from S3 (format: "s3://bucket/group-uuid/file-uuid.ext")
pub async fn download_from_s3(s3: &S3Instance, s3_id: &str) -> Result<Vec<u8>, String> {
  let (group, file, ext) = parse_s3_id(s3_id)?;
  let key = format!("/{}/{}.{}", group, file, ext);

  let mut bucket: Box<Bucket> =
    Bucket::new(&s3.bucket_name, s3.region.clone(), s3.credentials.clone())
      .map_err(|e| format!("Failed to create S3 bucket: {:?}", e))?;

  bucket.set_path_style();
  let response = bucket
    .get_object(key)
    .await
    .map_err(|e| format!("S3 download failed: {}", e))?;

  Ok(response.bytes().to_vec())
}

// Upload filtered CSV bytes to S3 with a new UUID
pub async fn upload_to_s3(s3: &S3Instance, content: &[u8], group: &Uuid) -> Result<String, String> {
  let new_file_uuid = Uuid::new_v4();
  let key = format!("/{}/{}.csv", group, new_file_uuid);

  let mut bucket: Box<Bucket> =
    Bucket::new(&s3.bucket_name, s3.region.clone(), s3.credentials.clone())
      .map_err(|e| format!("Failed to create S3 bucket: {:?}", e))?;

  bucket.set_path_style();
  bucket
    .put_object(&key, content)
    .await
    .map_err(|e| format!("S3 upload failed: {}", e))?;

  Ok(format!(
    "s3://{}/{}/{}.csv",
    s3.bucket_name, group, new_file_uuid
  ))
}

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
    .position(|h| h == column)
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

pub fn parse_s3_id(s3_id: &str) -> Result<(Uuid, Uuid, String), String> {
  let parts: Vec<&str> = s3_id.split('/').collect();
  if parts.len() < 4 {
    return Err(format!("Invalid s3_id format: {}", s3_id));
  }

  let group = Uuid::parse_str(parts[parts.len() - 2])
    .map_err(|e| format!("Invalid group UUID in s3_id: {}", e))?;

  let last = parts[parts.len() - 1];
  let (file_str, ext) = last.split_once('.').unwrap_or((last, "csv"));
  let file = Uuid::parse_str(file_str).map_err(|e| format!("Invalid file UUID in s3_id: {}", e))?;

  Ok((group, file, ext.to_string()))
}
