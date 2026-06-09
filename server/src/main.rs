mod api;
mod errors;
mod handlers;
mod infra;
mod models;

use std::env;

use crate::api::router::router as app_router;
use crate::infra::redis::config::init_redis_connection;
use apalis_redis::RedisStorage;
use common::infra::database::config::create_pool_from_env;
use common::infra::s3::config::{S3Instance, init_s3_instance};
use common::queue::models::Job;
use common::queue::storage::get_queue_storage;
use redis::aio::ConnectionManager;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
  pub pool: Pool<Postgres>,
  pub s3_instance: S3Instance,
  pub storage: RedisStorage<Job>,
  pub redis_connection: ConnectionManager,
  pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
  let pool: Pool<Postgres> = create_pool_from_env().await.unwrap();
  let s3_instance: S3Instance = init_s3_instance();
  let storage: RedisStorage<Job> = get_queue_storage().await;
  let redis_connection = init_redis_connection()
    .await
    .expect("Failed to connect to redis.");
  let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");

  let state: AppState = AppState {
    pool,
    s3_instance,
    storage,
    redis_connection,
    jwt_secret,
  };

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  let router = app_router(state);
  axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install SIGTERM handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
