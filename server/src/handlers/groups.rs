use crate::AppState;
use crate::api::middleware::AuthenticatedUser;
use crate::errors::AppError;
use crate::infra::database::group::{
  add_user_to_group, count_groups_by_user, create_group, delete_group, get_groups_by_user,
  list_group_members, search_groups_excluding_user,
};
use crate::models::group::{
  CreateGroupRequest, GroupResponse, MemberResponse, PaginatedGroupsResponse, PaginationParams,
  SearchParams,
};
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use uuid::Uuid;

pub async fn create_group_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), AppError> {
  let group = create_group(&state.pool, &payload.name)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        return AppError::Conflict("Group name already taken");
      }
      AppError::Internal("Error creating group")
    })?;

  // Auto-join the creator
  add_user_to_group(&state.pool, claims.sub, group.id)
    .await
    .map_err(|_| AppError::Internal("Error adding creator to group"))?;

  Ok((StatusCode::CREATED, Json(GroupResponse::from(group))))
}

pub async fn join_group_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(group_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
  add_user_to_group(&state.pool, claims.sub, group_id)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        return AppError::Conflict("Already a member of this group");
      }
      AppError::Internal("Error joining group")
    })?;

  Ok(StatusCode::OK)
}

pub async fn list_members_handler(
  State(state): State<AppState>,
  Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
  let members = list_group_members(&state.pool, group_id)
    .await
    .map_err(|_| AppError::Internal("Error listing members"))?;

  Ok(Json(members))
}

pub async fn search_groups_handler(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
  if params.q.trim().is_empty() {
    return Ok(Json(Vec::<GroupResponse>::new()));
  }

  let groups = search_groups_excluding_user(&state.pool, claims.sub, &params.q)
    .await
    .map_err(|_| AppError::Internal("Error searching groups"))?;

  Ok(Json(
    groups
      .into_iter()
      .map(GroupResponse::from)
      .collect::<Vec<_>>(),
  ))
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
  AuthenticatedUser(claims): AuthenticatedUser,
  Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
  let page = params.page.max(1);
  let page_size = params.page_size.clamp(1, 50);

  let (groups, total) = tokio::try_join!(
    get_groups_by_user(&state.pool, claims.sub, page, page_size),
    count_groups_by_user(&state.pool, claims.sub),
  )
  .map_err(|_| AppError::Internal("Error retrieving groups"))?;

  Ok(Json(PaginatedGroupsResponse {
    groups: groups.into_iter().map(GroupResponse::from).collect(),
    total,
  }))
}
