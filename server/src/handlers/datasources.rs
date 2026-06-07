use apalis_core::request::Parts;
use apalis_redis::{RedisContext, RedisStorage};
use axum::{
  Json,
  body::Bytes,
  extract::{Multipart, Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use common::infra::database::datasource::{Datasource, DatasourceType};
use common::queue::models::{Job, Op, Pipeline};
use csv::Reader;
use s3::{Bucket, error::S3Error};
use serde_json::Value;
use sqlx::{Error, Pool, Postgres, Row, postgres::PgRow, query};
use std::str::FromStr;
use uuid::Uuid;

use crate::AppState;
use crate::S3Instance;
use crate::handlers::jobs::{add_job_to_postgres, add_job_to_redis};

pub async fn get_all_datasources(
  State(state): State<AppState>,
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
    .fetch_all(&state.pool)
    .await;

  match datasources {
    Ok(dt) => Ok(Json(dt)),
    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
  }
}

async fn fetch_datasource(
  db_pool: &Pool<Postgres>,
  id: &Uuid,
) -> Result<Option<Datasource>, sqlx::Error> {
  // TODO: authentication and authorization checks should be implemented here

  sqlx::query_as::<_, Datasource>(
    "SELECT id, s3_id, name, file_type, size, created_at, group_id FROM datasources WHERE id = $1",
  )
  .bind(id)
  .fetch_optional(db_pool)
  .await
}

pub async fn get_datasource_by_id(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<Json<Datasource>, (StatusCode, String)> {
  // TODO: authentication and authorization checks should be implemented here

  match fetch_datasource(&state.pool, &id).await {
    Ok(Some(dt)) => Ok(Json(dt)),
    Ok(None) => Err((StatusCode::NOT_FOUND, "Datasource not found".to_string())),
    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
  }
}

struct Metadata {
  file_type: DatasourceType,
  header: Option<String>,
}

struct FileUploadRequest {
  file_content: Bytes,
  file_name: String,
  file_size: f64,
  metadata: Metadata,
}

fn parse_metadata(metadata: Value) -> Result<Metadata, (StatusCode, String)> {
  let file_type: DatasourceType = match metadata.get("type") {
    Some(val) => {
      let clean = val.as_str().unwrap_or("").trim_matches('"');
      match DatasourceType::from_str(clean) {
        Ok(t) => t,
        Err(_) => {
          return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid 'type' value: {}", val),
          ));
        }
      }
    }

    None => {
      return Err((
        StatusCode::BAD_REQUEST,
        "Missing 'type' field in metadata".to_string(),
      ));
    }
  };

  let mut header: Option<String> = None;
  if file_type == DatasourceType::Csv {
    header = match metadata.get("header") {
      Some(val) => Some(val.to_string().trim_matches('"').parse().map_err(|_| {
        (
          StatusCode::BAD_REQUEST,
          "Cannot convert 'header' field to boolean".to_string(),
        )
      })?),
      None => {
        return Err((
          StatusCode::BAD_REQUEST,
          "Missing 'header' field in metadata".to_string(),
        ));
      }
    };
  }

  Ok(Metadata { file_type, header })
}

async fn parse_multipart(
  mut multipart: Multipart,
) -> Result<FileUploadRequest, (StatusCode, String)> {
  // Initiate needed variables from request
  let mut file_content: Option<Bytes> = None;
  let mut file_name: Option<String> = None;
  let mut metadata: Option<Value> = None;

  // Try to read body request
  while let Some(field) = multipart.next_field().await.map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      format!("Failed to read multipart: {:?}", e),
    )
  })? {
    let key = field.name().unwrap_or("").to_string();
    match key.as_str() {
      "file" => {
        file_name = Some(
          field
            .file_name()
            .ok_or((StatusCode::BAD_REQUEST, "Missing file name".to_string()))?
            .to_string(),
        );
        file_content = Some(field.bytes().await.map_err(|e| {
          (
            StatusCode::BAD_REQUEST,
            format!("Failed to read file: {:?}", e),
          )
        })?);
      }
      "metadata" => {
        let text = field.text().await.map_err(|e| {
          (
            StatusCode::BAD_REQUEST,
            format!("Failed to read metadata: {:?}", e),
          )
        })?;
        metadata = Some(serde_json::from_str(&text).map_err(|e| {
          (
            StatusCode::BAD_REQUEST,
            format!("Invalid metadata JSON: {:?}", e),
          )
        })?);
      }
      _ => {}
    }
  }

  // Return BAD_REQUEST if any field is missing
  let file_content: Bytes =
    file_content.ok_or((StatusCode::BAD_REQUEST, "Missing file field".to_string()))?;
  let file_name: String =
    file_name.ok_or((StatusCode::BAD_REQUEST, "Missing file name".to_string()))?;
  let metadata: Value = metadata.ok_or((
    StatusCode::BAD_REQUEST,
    "Missing metadata field".to_string(),
  ))?;
  let file_size: f64 = file_content.len() as f64 / (1024.0 * 1024.0); // Convert bytes to MB

  let parsed_metadata = parse_metadata(metadata)?;

  Ok(FileUploadRequest {
    file_content,
    file_name,
    file_size,
    metadata: parsed_metadata,
  })
}

fn create_pipeline(metadata: &Metadata) -> Result<Pipeline, String> {
  Ok(vec![Op::Ingest {
    r#type: metadata.file_type,
    header: metadata.header.clone(),
  }])
}

fn validate_csv(content: &Bytes) -> Result<(), String> {
  let mut reader = Reader::from_reader(content.as_ref());

  for result in reader.records() {
    result.map_err(|e| format!("Invalid CSV: {:?}", e))?;
  }

  Ok(())
}

fn validate_json(content: &Bytes) -> Result<(), String> {
  let content_string =
    str::from_utf8(content).map_err(|e| format!("Cannot parse JSON: {:?}", e))?;
  serde_json::from_str::<Value>(content_string)
    .map(|_| ())
    .map_err(|e| format!("Invalid JSON: {:?}", e))
}

fn validate_file_format(content: &Bytes, file_type: &DatasourceType) -> Result<(), String> {
  match file_type {
    DatasourceType::Csv => validate_csv(content),
    DatasourceType::Json => validate_json(content),
  }
}

async fn add_file_to_s3(
  s3_instance: &S3Instance,
  file_content: &Bytes,
  file_uuid: &Uuid,
  file_format: &DatasourceType,
  group: &Uuid,
) -> Result<String, S3Error> {
  let mut bucket: Box<Bucket> = Bucket::new(
    &s3_instance.bucket_name,
    s3_instance.region.clone(),
    s3_instance.credentials.clone(),
  )
  .unwrap();

  let s3_key = format!("/{}/{}.{:?}", group, file_uuid, file_format).to_lowercase();
  bucket.set_path_style();
  bucket.put_object(&s3_key, file_content).await?;

  Ok(format!("s3://{}{}", s3_instance.bucket_name, s3_key))
}

async fn add_datasource_to_postgres(
  pool: &Pool<Postgres>,
  file_uuid: &Uuid,
  datasource_s3_id: &str,
  file_name: &str,
  file_type: &DatasourceType,
  file_size: &f64,
  group: &Uuid,
) -> Result<PgRow, Error> {
  query(
    "
  INSERT INTO
  datasources (id, s3_id, name, file_type, size, group_id)
  VALUES ($1, $2, $3, $4, $5, $6 ) RETURNING *;",
  )
  .bind(file_uuid)
  .bind(datasource_s3_id)
  .bind(file_name)
  .bind(file_type)
  .bind(file_size)
  .bind(group)
  .fetch_one(pool)
  .await
}

pub async fn csv_ingestion_handler(
  State(state): State<AppState>,
  multipart: Multipart,
) -> impl IntoResponse {
  // Handle body request
  let FileUploadRequest {
    file_content,
    file_name,
    file_size,
    metadata,
  } = match parse_multipart(multipart).await {
    Ok(val) => val,
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
  };

  let pipeline = match create_pipeline(&metadata) {
    Ok(p) => p,
    Err(e) => {
      return (
        StatusCode::BAD_REQUEST,
        format!("Error creating pipeline: {}", e),
      );
    }
  };

  // Make sure file is a correct format
  if let Err(e) = validate_file_format(&file_content, &metadata.file_type) {
    return (StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
  }

  // TO-DO : Implement authentication and change group dynamically
  let group: Uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

  // Generate File and Job UUID
  let file_uuid: Uuid = Uuid::new_v4();
  let job_uuid: Uuid = Uuid::new_v4();

  // Upload file to S3 Bucket
  let s3_instance: S3Instance = state.s3_instance;
  let datasource_s3_id: String = match add_file_to_s3(
    &s3_instance,
    &file_content,
    &file_uuid,
    &metadata.file_type,
    &group,
  )
  .await
  {
    Ok(s3_id) => s3_id,
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
  };

  // Add datasource to Postgres
  let pool: Pool<Postgres> = state.pool;
  let datasource_to_postgres: Result<PgRow, Error> = add_datasource_to_postgres(
    &pool,
    &file_uuid,
    &datasource_s3_id,
    &file_name,
    &metadata.file_type,
    &file_size,
    &group,
  )
  .await;

  if let Err(e) = datasource_to_postgres {
    return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e));
  };

  // Define job name
  let job_name: String = format!("Ingestion of {}", &file_name);

  // Add ingest job to Redis
  let redis_conn: RedisStorage<Job> = state.storage;
  let ingest_job_to_redis: Result<Parts<RedisContext>, Error> = add_job_to_redis(
    &redis_conn,
    &pipeline,
    &job_uuid,
    &job_name,
    &datasource_s3_id,
  )
  .await;

  if let Err(e) = ingest_job_to_redis {
    return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e));
  };

  // Add ingest job to Postgres
  let ingest_job_to_postgres: Result<(), Error> =
    add_job_to_postgres(&pool, &pipeline, &job_uuid, &job_name, &file_uuid).await;

  if let Err(e) = ingest_job_to_postgres {
    return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e));
  };

  (StatusCode::OK, "Upload file successful.".to_string())
}

pub async fn delete_datasource_by_id(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> impl IntoResponse {
  // TODO: authentication and authorization checks should be implemented here

  let datasource = match fetch_datasource(&state.pool, &id).await {
    Ok(Some(ds)) => ds,
    Ok(None) => return (StatusCode::NOT_FOUND, "Datasource not found".to_string()),
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
  };

  // Check active jobs
  let has_active_jobs = match query(
    "SELECT EXISTS(
      SELECT 1 FROM jobs j
      JOIN job_datasources jd ON j.id = jd.job_id
      WHERE jd.datasource_id = $1 AND j.status IN ('pending', 'running')
    ) as is_active",
  )
  .bind(id)
  .fetch_one(&state.pool)
  .await
  {
    Ok(row) => row.get::<bool, _>("is_active"),
    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
  };

  if has_active_jobs {
    return (
      StatusCode::CONFLICT,
      "Cannot delete datasource with active jobs".to_string(),
    );
  }

  // Delete from S3
  if let Err(e) = delete_file_from_s3(&state.s3_instance, &datasource.s3_id).await {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("S3 deletion failed: {:?}", e),
    );
  }

  // Delete from Postgres
  if let Err(e) = delete_datasource_from_postgres(&state.pool, &id).await {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Database deletion failed: {:?}", e),
    );
  }

  (
    StatusCode::OK,
    "Datasource deleted successfully".to_string(),
  )
}

pub async fn delete_datasource_from_postgres(
  pool: &Pool<Postgres>,
  id: &Uuid,
) -> Result<(), Error> {
  query(" DELETE FROM datasources WHERE id = $1;")
    .bind(id)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn delete_file_from_s3(s3_instance: &S3Instance, s3_id: &str) -> Result<(), S3Error> {
  let key = s3_id
    .strip_prefix("s3://")
    .and_then(|s| s.splitn(2, '/').nth(1))
    .map(|p| format!("/{}", p))
    .unwrap_or_default();

  let mut bucket: Box<Bucket> = Bucket::new(
    &s3_instance.bucket_name,
    s3_instance.region.clone(),
    s3_instance.credentials.clone(),
  )
  .unwrap();
  bucket.set_path_style();
  bucket.delete_object(key).await.map(|_| ())
}
