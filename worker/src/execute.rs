use common::infra::s3::config::S3Instance;
use common::queue::models::{Job, Op};
use uuid::Uuid;

use crate::filter;

pub trait Operation {
  async fn execute(
    &self,
    job: &Job,
    s3: &S3Instance,
  ) -> Result<(String, String, Uuid, f64), String>;
}

impl Operation for Op {
  async fn execute(
    &self,
    job: &Job,
    s3: &S3Instance,
  ) -> Result<(String, String, Uuid, f64), String> {
    match self {
      Op::Filter {
        column,
        operator,
        value,
      } => filter::execute(job, column, operator, value, s3).await,
      _ => Err("Operation not implemented in worker".into()),
    }
  }
}
