use csv::{Reader, Writer};
use std::collections::BTreeMap;

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
