use crate::AppState;
use crate::handlers::datasources::csv_ingestion_handler;
use crate::handlers::datasources::*;
use axum::{
  Router,
  http::StatusCode,
  routing::{delete, get, post},
};

fn datasources_router() -> Router<AppState> {
  Router::new()
    .route("/", get(get_all_datasources))
    .route("/{id}", get(get_datasource_by_id))
    .route("/", post(csv_ingestion_handler))
    .route("/{id}", delete(|| async { StatusCode::NOT_IMPLEMENTED }))
}

fn jobs_router() -> Router<AppState> {
  Router::new()
    .route("/", post(|| async { StatusCode::NOT_IMPLEMENTED }))
    .route("/", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .route("/{id}", get(|| async { StatusCode::NOT_IMPLEMENTED }))
}

pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .nest("/datasources", datasources_router())
    .nest("/jobs", jobs_router())
    .with_state(state)
}
