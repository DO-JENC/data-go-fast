use apalis_core::request::Parts;
use apalis_redis::{RedisContext, RedisStorage};
use axum::{
  Json,
  body::Bytes,
  extract::{Multipart, Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use common::{
  infra::database::datasource::{Datasource, DatasourceType, create_datasource_from_s3},
  queue::models::{Job, Op, Pipeline},
};
use csv::Reader;
use s3::{Bucket, error::S3Error};
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Error, Pool, Postgres, Row, query};
use std::str::FromStr;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::S3Instance;
use crate::{
  AppState,
  handlers::jobs::{add_job_to_postgres, add_job_to_redis},
};

#[derive(Deserialize, Debug)]
pub struct DatasourceFilters {
  pub group_id: Option<Uuid>,
  pub limit: Option<i64>,
  pub offset: Option<i64>,
}

#[derive(serde::Serialize, Debug)]
pub struct PaginatedResponse<T> {
  pub items: Vec<T>,
  pub total: i64,
}

#[instrument(skip(state))]
pub async fn get_all_datasources(
  State(state): State<AppState>,
  Query(filters): Query<DatasourceFilters>,
) -> Result<Json<PaginatedResponse<Datasource>>, (StatusCode, String)> {
  let group_id = filters.group_id.ok_or_else(|| {
    warn!("get_all_datasources called without group_id");
    (
      StatusCode::BAD_REQUEST,
      "Missing required parameter: group_id".to_string(),
    )
  })?;

  let limit = filters.limit.unwrap_or(10);
  let offset = filters.offset.unwrap_or(0);

  let total_query = "SELECT COUNT(*) FROM datasources WHERE group_id = $1";
  let total: i64 = sqlx::query_scalar(total_query)
    .bind(group_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
      error!("Failed to fetch total datasources: {:?}", e);
      (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

  let items_query = r#"
        SELECT id, s3_id, name, file_type, size, created_at, group_id
        FROM datasources
        WHERE group_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
  "#;

  let datasources = sqlx::query_as::<_, Datasource>(items_query)
    .bind(group_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
      error!("Failed to fetch datasources: {:?}", e);
      (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

  Ok(Json(PaginatedResponse {
    items: datasources,
    total,
  }))
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

#[instrument(skip(state))]
pub async fn get_datasource_by_id(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<Json<Datasource>, (StatusCode, String)> {
  // TODO: authentication and authorization checks should be implemented here

  match fetch_datasource(&state.pool, &id).await {
    Ok(Some(dt)) => Ok(Json(dt)),
    Ok(None) => {
      warn!("Datasource not found: {}", id);
      Err((StatusCode::NOT_FOUND, "Datasource not found".to_string()))
    }
    Err(e) => {
      error!("Failed to fetch datasource {}: {:?}", id, e);
      Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
  }
}

struct Metadata {
  file_type: DatasourceType,
  header: bool,
  group_id: Uuid,
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

  let header = metadata
    .get("header")
    .map(|v| {
      v.to_string()
        .trim_matches('"')
        .parse::<bool>()
        .unwrap_or(true)
    })
    .unwrap_or(true);

  let group_id: Uuid = match metadata.get("group_id") {
    Some(val) => {
      let raw = val.as_str().unwrap_or("").trim_matches('"');
      Uuid::parse_str(raw).map_err(|_| {
        (
          StatusCode::BAD_REQUEST,
          format!("Invalid 'group_id' value: {}", val),
        )
      })?
    }
    None => {
      return Err((
        StatusCode::BAD_REQUEST,
        "Missing 'group_id' field in metadata".to_string(),
      ));
    }
  };

  Ok(Metadata {
    file_type,
    header,
    group_id,
  })
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

fn create_pipeline(metadata: &Metadata) -> Result<Pipeline, String> {
  Ok(vec![Op::Ingest {
    r#type: metadata.file_type,
    header: Some(metadata.header.to_string()).is_some(),
  }])
}

#[instrument(skip(state, multipart))]
pub async fn csv_ingestion_handler(
  State(state): State<AppState>,
  multipart: Multipart,
) -> impl IntoResponse {
  info!("Starting file ingestion");
  // Handle body request
  let FileUploadRequest {
    file_content,
    file_name,
    file_size,
    metadata,
  } = match parse_multipart(multipart).await {
    Ok(val) => val,
    Err(e) => {
      warn!("Failed to parse multipart: {:?}", e);
      return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e));
    }
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
  info!("Validating file: {} (size: {:.2} MB)", file_name, file_size);

  // Make sure file is a correct format
  if let Err(e) = validate_file_format(&file_content, &metadata.file_type) {
    warn!("File validation failed for {}: {}", file_name, e);
    return (StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
  }

  if metadata.file_type == DatasourceType::Csv && !metadata.header {
    warn!("Headerless CSV rejected for {}", file_name);
    return (
      StatusCode::BAD_REQUEST,
      "Headerless CSV files are not yet supported".to_string(),
    );
  }

  let group: Uuid = metadata.group_id;

  // Generate File & Job UUID
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
    Ok(s3_id) => {
      info!("File uploaded to S3: {}", s3_id);
      s3_id
    }
    Err(e) => {
      error!("Failed to upload file to S3: {:?}", e);
      return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e));
    }
  };

  // Add datasource to Postgres
  let pool: Pool<Postgres> = state.pool;
  let datasource_to_postgres = create_datasource_from_s3(
    &pool,
    &datasource_s3_id,
    &file_name,
    &group,
    file_size,
    metadata.file_type,
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

  match datasource_to_postgres {
    Ok(_) => {
      info!("Datasource {} added to database", file_uuid);
      (StatusCode::OK, "Upload file successful.".to_string())
    }
    Err(e) => {
      error!("Failed to add datasource to database: {:?}", e);
      (StatusCode::BAD_REQUEST, format!("Error: {:?}", e))
    }
  }
}

#[instrument(skip(state))]
pub async fn delete_datasource_by_id(
  State(state): State<AppState>,
  Path(id): Path<Uuid>,
) -> impl IntoResponse {
  info!("Attempting to delete datasource: {}", id);

  let datasource = match fetch_datasource(&state.pool, &id).await {
    Ok(Some(ds)) => ds,
    Ok(None) => {
      warn!("Datasource not found: {}", id);
      return (StatusCode::NOT_FOUND, "Datasource not found".to_string());
    }
    Err(e) => {
      error!("Failed to fetch datasource for deletion: {:?}", e);
      return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
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
    Err(e) => {
      error!("Failed to check active jobs for datasource: {:?}", e);
      return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
  };

  if has_active_jobs {
    warn!("Cannot delete datasource {} with active jobs", id);
    return (
      StatusCode::CONFLICT,
      "Cannot delete datasource with active jobs".to_string(),
    );
  }

  // Delete from S3
  if let Err(e) = delete_file_from_s3(&state.s3_instance, &datasource.s3_id).await {
    error!("S3 deletion failed for {}: {:?}", datasource.s3_id, e);
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("S3 deletion failed: {:?}", e),
    );
  }
  info!("Deleted from S3: {}", datasource.s3_id);

  // Delete from Postgres
  if let Err(e) = delete_datasource_from_postgres(&state.pool, &id).await {
    error!("Database deletion failed for datasource {}: {:?}", id, e);
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Database deletion failed: {:?}", e),
    );
  }
  info!("Deleted from database: {}", id);

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
    .and_then(|s| s.split_once('/').map(|(_, p)| format!("/{}", p)))
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
