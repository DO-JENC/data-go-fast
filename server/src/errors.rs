use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
  Unauthorized(&'static str),
  Conflict(&'static str),
  Internal(&'static str),
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, message) = match self {
      AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
      AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
      AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };
    (status, Json(json!({ "error": message }))).into_response()
  }
}
