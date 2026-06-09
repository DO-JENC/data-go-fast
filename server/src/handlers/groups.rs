use crate::AppState;
use crate::errors::AppError;
use crate::infra::database::group::{
  add_user_to_group, create_group, delete_group, get_groups, list_group_members,
};
use crate::models::group::{CreateGroupRequest, GroupResponse, JoinGroupRequest, MemberResponse};
use axum::response::IntoResponse;
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use uuid::Uuid;

pub async fn create_group_handler(
  State(state): State<AppState>,
  Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), (StatusCode, String)> {
  let group = create_group(&state.pool, &payload.name)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        return (StatusCode::CONFLICT, "Group already exists".to_string());
      }
      (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

  Ok((StatusCode::CREATED, Json(GroupResponse::from(group))))
}

pub async fn join_group_handler(
  State(state): State<AppState>,
  Path(group_id): Path<Uuid>,
  Json(payload): Json<JoinGroupRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
  add_user_to_group(&state.pool, payload.user_id, group_id)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        return (StatusCode::CONFLICT, "User already in group".to_string());
      }
      (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

  Ok(StatusCode::OK)
}

pub async fn list_members_handler(
  State(state): State<AppState>,
  Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, (StatusCode, String)> {
  let members = list_group_members(&state.pool, group_id)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  Ok(Json(members))
}

pub async fn delete_group_handler(
  State(state): State<AppState>,
  Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
  delete_group(&state.pool, group_id)
    .await
    .map_err(|_| AppError::Internal("Error deleting group"))?;

  Ok(StatusCode::NO_CONTENT)
}

pub async fn get_groups_handler(
  State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
  let groups = get_groups(&state.pool)
    .await
    .map_err(|_| AppError::Internal("Error retrieving groups from database"))?;

  Ok(Json(groups))
}
