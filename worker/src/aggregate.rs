use csv::Reader;
use serde_json::{Map, Value};
use std::collections::HashMap;

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
