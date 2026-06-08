use crate::AppState;
use crate::handlers::datasources::*;
use crate::handlers::groups::*;
use crate::handlers::jobs::*;
use crate::handlers::users::*;
use axum::extract::DefaultBodyLimit;
use axum::{
  Router,
  http::StatusCode,
  routing::{delete, get, post},
};

const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1GB

fn datasources_router() -> Router<AppState> {
  Router::new()
    .route("/", get(get_all_datasources))
    .route("/{id}", get(get_datasource_by_id))
    .route("/", post(csv_ingestion_handler))
    .route("/{id}", delete(delete_datasource_by_id))
    .layer(DefaultBodyLimit::max(FILE_SIZE_LIMIT))
}

fn jobs_router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_job_handler))
    .route("/", get(list_jobs_handler))
    .route("/{id}", get(get_job_by_id_handler))
}

fn users_router() -> Router<AppState> {
  Router::new()
    .route("/signup", post(signup))
    .route("/", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .route("/{id}", get(|| async { StatusCode::NOT_IMPLEMENTED }))
}

fn groups_router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_group_handler))
    .route("/{id}/join", post(join_group_handler))
    .route("/{id}/members", get(list_members_handler))
}
pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .nest("/datasources", datasources_router())
    .nest("/jobs", jobs_router())
    .nest("/users", users_router())
    .nest("/groups", groups_router())
    .with_state(state)
}
