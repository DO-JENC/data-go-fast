use crate::infra::database::datasource::DatasourceType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
  pub job_id: Uuid,
  pub datasource_id: String,
  pub name: String,
  pub pipeline: Pipeline,
  pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Pipeline {
  pub op: String,
  pub r#type: DatasourceType,
  pub header: Option<String>,
}
