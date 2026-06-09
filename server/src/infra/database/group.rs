use crate::models::group::{Group, MemberResponse};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_groups_by_user(
  pool: &PgPool,
  user_id: Uuid,
  page: i64,
  page_size: i64,
) -> Result<Vec<Group>, sqlx::Error> {
  let offset = (page - 1) * page_size;
  sqlx::query_as::<_, Group>(
    "SELECT g.id, g.name
         FROM groups g
         JOIN user_groups ug ON g.id = ug.group_id
         WHERE ug.user_id = $1
         ORDER BY g.name
         LIMIT $2 OFFSET $3",
  )
  .bind(user_id)
  .bind(page_size)
  .bind(offset)
  .fetch_all(pool)
  .await
}

pub async fn count_groups_by_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
  let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_groups WHERE user_id = $1")
    .bind(user_id)
    .fetch_one(pool)
    .await?;
  Ok(row.0)
}

pub async fn search_groups_excluding_user(
  pool: &PgPool,
  user_id: Uuid,
  query: &str,
) -> Result<Vec<Group>, sqlx::Error> {
  let pattern = format!("%{}%", query.to_lowercase());
  sqlx::query_as::<_, Group>(
    "SELECT id, name FROM groups
         WHERE LOWER(name) LIKE $1
           AND id NOT IN (
               SELECT group_id FROM user_groups WHERE user_id = $2
           )
         ORDER BY name
         LIMIT 20",
  )
  .bind(pattern)
  .bind(user_id)
  .fetch_all(pool)
  .await
}

pub async fn create_group(pool: &PgPool, name: &str) -> Result<Group, sqlx::Error> {
  let group =
    sqlx::query_as::<_, Group>("INSERT INTO groups (name) VALUES ($1) RETURNING id, name")
      .bind(name)
      .fetch_one(pool)
      .await?;

  Ok(group)
}
pub async fn delete_group(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
  sqlx::query("DELETE FROM groups WHERE id=$1")
    .bind(id)
    .execute(pool)
    .await?;

  Ok(())
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
