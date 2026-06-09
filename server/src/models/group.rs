use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Group {
  pub id: Uuid,
  pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedGroupsResponse {
  pub groups: Vec<GroupResponse>,
  pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
  pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
  #[serde(default = "default_page")]
  pub page: i64,
  #[serde(default = "default_page_size")]
  pub page_size: i64,
}

fn default_page() -> i64 {
  1
}
fn default_page_size() -> i64 {
  5
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
  pub name: String,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
  pub id: Uuid,
  pub name: String,
}

impl From<Group> for GroupResponse {
  fn from(group: Group) -> Self {
    Self {
      id: group.id,
      name: group.name,
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct JoinGroupRequest {
  pub user_id: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MemberResponse {
  pub id: Uuid,
  pub email: String,
}
