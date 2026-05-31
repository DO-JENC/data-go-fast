use crate::models::group::{Group, MemberResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_group(pool: &PgPool, name: &str) -> Result<Group, sqlx::Error> {
  let group =
    sqlx::query_as::<_, Group>("INSERT INTO groups (name) VALUES ($1) RETURNING id, name")
      .bind(name)
      .fetch_one(pool)
      .await?;

  Ok(group)
}

pub async fn add_user_to_group(
  pool: &PgPool,
  user_id: Uuid,
  group_id: Uuid,
) -> Result<(), sqlx::Error> {
  sqlx::query("INSERT INTO user_groups (user_id, group_id) VALUES ($1, $2)")
    .bind(user_id)
    .bind(group_id)
    .execute(pool)
    .await?;

  Ok(())
}

pub async fn list_group_members(
  pool: &PgPool,
  group_id: Uuid,
) -> Result<Vec<MemberResponse>, sqlx::Error> {
  let members = sqlx::query_as::<_, MemberResponse>(
    "SELECT u.id, u.email FROM users u JOIN user_groups ug ON u.id = ug.user_id WHERE ug.group_id = $1",
  )
  .bind(group_id)
  .fetch_all(pool)
  .await?;

  Ok(members)
}
