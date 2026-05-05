use axum::{
  Router,
  routing::{delete, get, post},
};

fn datasources_router() -> Router {
  Router::new()
    .route("/", get("GET /datasources"))
    .route("/", post("POST /datasources"))
    .route("/{id}", get("GET /datasources/{id}"))
    .route("/{id}", delete("DELETE /datasources/{id}"))
}

fn jobs_router() -> Router {
  Router::new()
    .route("/", post("POST /jobs"))
    .route("/", get("GET /jobs"))
    .route("/{id}", get("GET /jobs/{id}"))
}

pub fn router() -> Router {
  Router::new()
    .nest("/datasources", datasources_router())
    .nest("/jobs", jobs_router())
}
