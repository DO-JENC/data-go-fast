use crate::{
  AppState, errors::AppError, infra::database::user::find_user_by_id,
  models::auth::AuthenticatedUser, models::user::UserResponse,
};
use axum::{Json, extract::State};

pub async fn get_me(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
  let user = find_user_by_id(&state.pool, claims.sub)
    .await
    .map_err(|_| AppError::Internal("Failed to fetch user"))?
    .ok_or(AppError::Unauthorized("User not found"))?;

  Ok(Json(UserResponse::from(user)))
}
