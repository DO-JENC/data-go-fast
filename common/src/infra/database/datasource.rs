use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
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
