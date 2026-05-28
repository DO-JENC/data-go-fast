mod api;
mod handlers;
mod infra;
mod models;
use std::env;

use crate::api::router::router as app_router;
use common::infra::database::config::create_pool_from_env;
use common::infra::s3::config::{S3Instance, init_s3_instance};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
  pub pool: Pool<Postgres>,
  pub s3_instance: S3Instance,
}

#[tokio::main]
async fn main() {
  let pool: Pool<Postgres> = create_pool_from_env().await.unwrap();
  let s3_instance = init_s3_instance();

  let state: AppState = AppState { pool, s3_instance };

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  let router = app_router(state);
  axum::serve(listener, router).await.unwrap();
}
