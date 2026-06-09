use csv::{Reader, Writer};
use serde_json::Value;

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
