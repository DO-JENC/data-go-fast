use common::queue::models::Op;

use crate::filter;

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
      _ => Err("Operation not implemented in worker".into()),
    }
  }
}
