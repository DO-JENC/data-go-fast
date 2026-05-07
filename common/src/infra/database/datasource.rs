use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, Debug, Clone)]
#[sqlx(type_name = "datasource_type", rename_all = "lowercase")]
pub enum DatasourceType {
  Csv,
  Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Datasource {
  pub id: Uuid,
  pub s3_id: Uuid,
  pub name: String,
  pub file_type: Option<DatasourceType>,
  pub size: f64,
  pub created_at: Option<DateTime<Utc>>,
  pub group_id: Option<Uuid>,
}
