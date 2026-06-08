use crate::{
  AppState,
  api::middleware::auth_middleware,
  handlers::{
    auth::{login, signup, refresh_token},
    datasources::*,
    groups::*,
    jobs::{create_job_handler, list_jobs_handler, get_job_by_id_handler},
    users::get_me,
  },
};

use axum::{
  Router,
  http::StatusCode,
  middleware,
  routing::{delete, get, post},
};

const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1GB
fn auth_router() -> Router<AppState> {
  Router::new()
    .route("/signup", post(signup))
    .route("/login", post(login))
    .route("/refresh", post(refresh_token))
}

fn datasources_router() -> Router<AppState> {
  Router::new()
    .route("/", get(get_all_datasources))
    .route("/{id}", get(get_datasource_by_id))
    .route("/", post(csv_ingestion_handler))
    .route("/{id}", delete(delete_datasource_by_id))
    .layer(DefaultBodyLimit::max(FILE_SIZE_LIMIT))
    .layer(middleware::from_fn(auth_middleware))
}

fn jobs_router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_job_handler))
    .route("/", get(list_jobs_handler))
    .route("/{id}", get(get_job_by_id_handler))
    .layer(middleware::from_fn(auth_middleware))
}

fn users_router() -> Router<AppState> {
  Router::new()
    .route("/me", get(get_me))
    .route("/", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .route("/{id}", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .layer(middleware::from_fn(auth_middleware))
}

fn groups_router() -> Router<AppState> {
  Router::new()
    .route("/", post(create_group_handler))
    .route("/{id}/join", post(join_group_handler))
    .route("/{id}/members", get(list_members_handler))
    .layer(middleware::from_fn(auth_middleware))
}
pub fn router(state: AppState) -> Router {
  Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .nest("/auth", auth_router())
    .nest("/datasources", datasources_router())
    .nest("/jobs", jobs_router())
    .nest("/users", users_router())
    .nest("/groups", groups_router())
    .with_state(state)
}
