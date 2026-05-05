mod api;

use crate::api::router::router as app_router;

#[tokio::main]
async fn main() {
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  let router = app_router();
  axum::serve(listener, router).await.unwrap();
}
