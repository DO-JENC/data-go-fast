use crate::{
  AppState, errors::AppError, infra::database::user::find_user_by_id,
  models::auth::AuthenticatedUser, models::user::UserResponse,
};
use axum::{Json, extract::State};
use tracing::{error, instrument};

#[instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn get_me(
  State(state): State<AppState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
  let user = find_user_by_id(&state.pool, claims.sub)
    .await
    .map_err(|e| {
      error!("Failed to fetch user {}: {:?}", claims.sub, e);
      AppError::Internal("Failed to fetch user")
    })?
    .ok_or_else(|| {
      error!("User {} not found in database", claims.sub);
      AppError::Unauthorized("User not found")
    })?;

  Ok(Json(UserResponse::from(user)))
}
