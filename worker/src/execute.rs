use common::queue::models::Op;

use crate::{aggregate, filter, group_by};

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
      Op::GroupBy { by, aggregate } => {
        group_by::group_by_csv(csv_bytes, by, &aggregate.column, &aggregate.function)
      }
      Op::Ingest { .. } => Err("Ingest is not handled in worker".into()),
    }
  }
}
