use common::queue::models::Op;

use crate::{aggregate, filter};

pub trait Operation {
  async fn execute_on_bytes(&self, csv_bytes: &[u8]) -> Result<Vec<u8>, String>;
}

impl Operation for Op {
  async fn execute_on_bytes(&self, csv_bytes: &[u8]) -> Result<Vec<u8>, String> {
    match self {
      Op::Filter {
        column,
        operator,
        value,
      } => filter::apply_filter(csv_bytes, column, operator, value),
      Op::Aggregate { columns, functions } => {
        aggregate::aggregate_csv(csv_bytes, columns, functions)
      }
      Op::Ingest { .. } => Err("Ingest is not handled in worker".into()),
    }
  }
}
