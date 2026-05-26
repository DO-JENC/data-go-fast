mod api;
mod handlers;
use std::env;

use crate::api::router::router as app_router;

#[tokio::main]
async fn main() {
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

  if env::var("DATABASE_URL").is_err() {
    panic!("DATABASE_URL environment variable is not set");
  }

  let pool = sqlx::PgPool::connect(&env::var("DATABASE_URL").unwrap())
    .await
    .unwrap();
  let router = app_router(pool);
  axum::serve(listener, router).await.unwrap();
}
