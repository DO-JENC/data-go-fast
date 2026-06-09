use crate::infra::database::datasource::DatasourceType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
  pub job_id: Uuid,
  pub datasource_id: String,
  pub name: String,
  pub pipeline: Pipeline,
  pub status: String,
  #[serde(default)]
  pub result_datasource_id: Option<Uuid>, // Set by the worker after running the pipeline
}

// Ordered list of operations applied sequentially to the datasource
pub type Pipeline = Vec<Op>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupByAggregate {
  pub column: String,
  pub function: String, // sum, avg, median, min, max, count
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// `tag = "op"` means the JSON key `"op"` determines which variant we deserialize to
#[serde(tag = "op")]
pub enum Op {
  #[serde(rename = "ingest")]
  Ingest {
    r#type: DatasourceType,
    header: Option<String>,
  },

  #[serde(rename = "filter")]
  Filter {
    column: String,
    operator: String, // > , < , >= , <= , == , !=
    value: Value,
  },

  #[serde(rename = "aggregate")]
  Aggregate {
    columns: Vec<String>,
    functions: Vec<String>, // [sum , avg , median , min , max , count]
  },

  #[serde(rename = "group_by")]
  GroupBy {
    by: String,
    aggregate: GroupByAggregate,
  },
}
