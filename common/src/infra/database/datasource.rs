use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use sqlx::{Pool, Postgres, query};
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

// Insert a datasource row for a file already uploaded to S3.
// Retries on UNIQUE_VIOLATION to safely handle concurrent jobs.
pub async fn create_datasource_from_s3(
  pool: &Pool<Postgres>,
  s3_id: &str,
  base_name: &str,
  group_id: &Uuid,
  size: f64,
  file_type: DatasourceType,
) -> Result<Uuid, sqlx::Error> {
  // Extract datasource ID from s3_id: "s3://bucket/group/<uuid>.ext"
  let id = s3_id
    .rsplit('/')
    .next()
    .and_then(|last| last.split('.').next())
    .and_then(|s| Uuid::parse_str(s).ok())
    .ok_or_else(|| sqlx::Error::Protocol("invalid s3_id format: missing UUID".into()))?;

  let mut suffix: i32 = -1;

  loop {
    suffix += 1;
    // Try base_name first, then base_name(1), base_name(2)...
    let name = if suffix == 0 {
      base_name.to_string()
    } else {
      format!("{} ({})", base_name, suffix)
    };

    let result = query(
      "INSERT INTO datasources (id, s3_id, name, file_type, size, group_id)
       VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(s3_id)
    .bind(&name)
    .bind(file_type)
    .bind(size)
    .bind(group_id)
    .execute(pool)
    .await;

    match result {
      Ok(_) => return Ok(id),
      // 23505 = PostgreSQL UNIQUE_VIOLATION → name was taken, retry with next suffix
      Err(sqlx::Error::Database(ref e)) if e.code().as_deref() == Some("23505") => continue,
      Err(e) => return Err(e),
    }
  }
}
