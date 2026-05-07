use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use common::infra::database::datasource::Datasource;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_all_datasources(
  State(pool): State<PgPool>,
) -> Result<Json<Vec<Datasource>>, (StatusCode, String)> {
  // TODO: use the connected user when authentication is implemented
  let group_id: Uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

  let query = r#"
        SELECT id, s3_id, name, file_type, size, created_at, group_id
        FROM datasources
        WHERE group_id = $1
  "#;

  let datasources = sqlx::query_as::<_, Datasource>(query)
    .bind(group_id)
    .fetch_all(&pool)
    .await;

  match datasources {
    Ok(dt) => Ok(Json(dt)),
    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
  }
}

pub async fn get_datasource_by_id(
  State(pool): State<PgPool>,
  Path(id): Path<Uuid>,
) -> Result<Json<Datasource>, (StatusCode, String)> {
  // TODO: authentication and authorization checks should be implemented here

  let query = r#"
        SELECT id, s3_id, name, file_type, size, created_at, group_id
        FROM datasources
        WHERE id = $1
  "#;

  let datasource = sqlx::query_as::<_, Datasource>(query)
    .bind(id)
    .fetch_optional(&pool)
    .await;

  match datasource {
    Ok(Some(dt)) => Ok(Json(dt)),
    Ok(None) => Err((StatusCode::NOT_FOUND, "Datasource not found".to_string())),
    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
  }
}
