use crate::{
  AppState,
  api::middleware::auth_middleware,
  handlers::{
    auth::{login, logout, refresh_token, signup},
    datasources::*,
    groups::*,
    jobs::*,
    users::*,
  },
};

use axum::{
  Router,
  extract::DefaultBodyLimit,
  http::{HeaderValue, Method, StatusCode},
  middleware,
  routing::{delete, get, post},
};
use tower_http::cors::{Any, CorsLayer};

const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 1024; // 1GB

fn auth_router() -> Router<AppState> {
  Router::new()
    .route("/signup", post(signup))
    .route("/login", post(login))
    .route("/refresh", post(refresh_token))
    .route("/logout", post(logout))
}

fn datasources_router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/", get(get_all_datasources))
    .route("/{id}", get(get_datasource_by_id))
    .route("/", post(csv_ingestion_handler))
    .route("/{id}", delete(delete_datasource_by_id))
    .layer(DefaultBodyLimit::max(FILE_SIZE_LIMIT))
    .layer(middleware::from_fn_with_state(state, auth_middleware))
}

fn jobs_router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/", post(create_job_handler))
    .route("/", get(list_jobs_handler))
    .route("/{id}", get(get_job_by_id_handler))
    .layer(middleware::from_fn_with_state(state, auth_middleware))
}

fn users_router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/me", get(get_me))
    .route("/", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .route("/{id}", get(|| async { StatusCode::NOT_IMPLEMENTED }))
    .layer(middleware::from_fn_with_state(state, auth_middleware))
}

fn groups_router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/", post(create_group_handler))
    .route("/{id}/join", post(join_group_handler))
    .route("/{id}/members", get(list_members_handler))
    .route("/", get(get_groups_handler))
    .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn router(state: AppState) -> Router {
  let cors = match std::env::var("ALLOWED_ORIGINS") {
    Ok(origins) if origins == "*" => CorsLayer::new()
      .allow_origin(Any)
      .allow_methods(Any)
      .allow_headers(Any),
    Ok(origins) => {
      let origins = origins
        .split(',')
        .map(|s| s.trim().parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();
      CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
          Method::GET,
          Method::POST,
          Method::PUT,
          Method::DELETE,
          Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_credentials(true)
    }
    Err(_) => CorsLayer::new()
      .allow_origin(Any)
      .allow_methods(Any)
      .allow_headers(Any),
  };

  Router::new()
    .route("/health", get(|| async { StatusCode::OK }))
    .nest("/auth", auth_router())
    .nest("/datasources", datasources_router(state.clone()))
    .nest("/jobs", jobs_router(state.clone()))
    .nest("/users", users_router(state.clone()))
    .nest("/groups", groups_router(state.clone()))
    .layer(cors)
    .with_state(state)
}
