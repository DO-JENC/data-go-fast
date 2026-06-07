use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use sqlx::{Pool, Postgres, Row, query};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[sqlx(type_name = "datasource_type", rename_all = "lowercase")]
pub enum DatasourceType {
  Csv,
  Json,
}

impl FromStr for DatasourceType {
  type Err = String;

  fn from_str(s: &str) -> Result<DatasourceType, Self::Err> {
    match s.to_lowercase().as_str() {
      "csv" => Ok(DatasourceType::Csv),
      "json" => Ok(DatasourceType::Json),
      _ => Err(format!("Unknown file type: {}", s)),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Datasource {
  pub id: Uuid,
  pub s3_id: String,
  pub name: String,
  pub file_type: Option<DatasourceType>,
  pub size: f64,
  pub created_at: Option<DateTime<Utc>>,
  pub group_id: Option<Uuid>,
}

// Insert a datasource row for a file already uploaded to S3
// Called by the worker after processing a filter/transform operation
pub async fn create_datasource_from_s3(
  pool: &Pool<Postgres>,
  s3_id: &str,
  name: &str,
  group_id: &Uuid,
  size: f64,
) -> Result<Uuid, sqlx::Error> {
  let id = Uuid::new_v4();
  query(
    "INSERT INTO datasources (id, s3_id, name, file_type, size, group_id)
     VALUES ($1, $2, $3, $4, $5, $6)",
  )
  .bind(id)
  .bind(s3_id)
  .bind(name)
  .bind(DatasourceType::Csv)
  .bind(size)
  .bind(group_id)
  .execute(pool)
  .await?;

  Ok(id)
}

// Generate a unique name like "file", "file(1)", "file(2)"...
pub async fn get_unique_datasource_name(
  pool: &Pool<Postgres>,
  base_name: &str,
  group_id: &Uuid,
) -> Result<String, sqlx::Error> {
  // Fetch all names that start with base_name (e.g. "sales_filtered%")
  let rows = query("SELECT name FROM datasources WHERE group_id = $1 AND name LIKE $2")
    .bind(group_id)
    .bind(format!("{}%", base_name))
    .fetch_all(pool)
    .await?;

  let existing: Vec<String> = rows.iter().map(|r| r.get::<String, _>("name")).collect();

  // Fast path: base_name is free
  if !existing.iter().any(|n| n == base_name) {
    return Ok(base_name.to_string());
  }

  // Find the lowest (n) not taken
  let mut i = 1;
  loop {
    let candidate = format!("{}({})", base_name, i);
    if !existing.iter().any(|n| n == &candidate) {
      return Ok(candidate);
    }
    i += 1;
  }
}
