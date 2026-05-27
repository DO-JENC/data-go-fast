mod api;
mod handlers;
use std::env;

use crate::api::router::router as app_router;

#[tokio::main]
async fn main() {
  let server_port = env::var("SERVER_PORT").expect("SERVER_PORT environment variable is not set");
  let addr = format!("0.0.0.0:{server_port}");
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

  let database_url =
    env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set");
  let pool = sqlx::PgPool::connect(&database_url).await.unwrap();

  let router = app_router(pool);
  axum::serve(listener, router).await.unwrap();
}
