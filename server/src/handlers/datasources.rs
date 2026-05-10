use common::infra::database::datasource::Datasource;
use sqlx::PgPool;
use uuid::Uuid;
use csv::Reader;
use awscreds::Credentials;
use axum::{Json, body::Bytes, extract::{Multipart, Path, State}, http::StatusCode, response::IntoResponse};
use redis::RedisResult;
use s3::{Bucket, error::S3Error, region::Region, request::ResponseData};
use serde_json::{Value, json};
use sqlx::{Error, Pool, Postgres, postgres::PgRow, query};
use uuid::Uuid;


use crate::AppState;
use crate::S3Instance;
use crate::handlers::jobs::{add_job_to_postgres, add_job_to_redis};

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

struct FileUploadRequest {
  file_content: Bytes,
  file_name: String,
  file_size: f64,
  metadata: Value,
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

  Ok(FileUploadRequest {
    file_content,
    file_name,
    file_size,
    metadata,
  })
}

fn validate_csv(content: &Bytes) -> Result<(), String> {
  let mut reader = Reader::from_reader(content.as_ref());

  for result in reader.records() {
    result.map_err(|e| format!("Invalid CSV: {:?}", e))?;
  }

  Ok(())
}

async fn add_file_to_s3(
  s3_instance: &S3Instance,
  file_content: &Bytes,
  file_uuid: &Uuid,
  group: &Uuid,
) -> Result<ResponseData, S3Error> {
  let mut bucket: Box<Bucket> = Bucket::new(
    &s3_instance.bucket_name,
    s3_instance.region.clone(),
    s3_instance.credentials.clone(),
  )
  .unwrap();

  // Add file to S3 bucket
  bucket.set_path_style();
  bucket
    .put_object(format!("/{}/{}.csv", group, file_uuid), &file_content)
    .await
}

async fn add_datasource_to_postgres(
  pool: &Pool<Postgres>,
  file_uuid: &Uuid,
  datasource_s3_id: &str,
  file_name: &str,
  file_size: &f64,
  group: &Uuid,
) -> Result<PgRow, Error> {
  query(
    "
  INSERT INTO
  datasources (id, s3_id, name, type, size, group_id)
  VALUES ($1, $2, $3, 'csv', $4, $5 ) RETURNING *;",
  )
  .bind(file_uuid)
  .bind(datasource_s3_id)
  .bind(file_name)
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

  // Initiate ingest job pipeline
  let file_type: &Value = match metadata.get("type") {
    Some(val) => val,
    None => {
      return (
        StatusCode::BAD_REQUEST,
        "Missing 'type' field in metadata".to_string(),
      );
    }
  };

  let header: &Value = match metadata.get("header") {
    Some(val) => val,
    None => {
      return (
        StatusCode::BAD_REQUEST,
        "Missing 'header' field in metadata".to_string(),
      );
    }
  };

  let pipeline: Value = json!([{
      "op": "ingest",
      "type": file_type,
      "header": header,
  }]);

  // Make sure file is a correct CSV
  if let Err(e) = validate_csv(&file_content) {
    return (StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
  }

  // TO-DO : Implement authentication and change group dynamically
  let group: Uuid = Uuid::parse_str("9251e420-2dd3-4776-9f53-98ad52e02d79").unwrap();

  // Generate File and Job UUID
  let file_uuid: Uuid = Uuid::new_v4();
  let job_uuid: Uuid = Uuid::new_v4();

  // Upload file to S3 Bucket
  let s3_instance: S3Instance = state.s3_instance;
  let file_to_s3: Result<ResponseData, S3Error> =
    add_file_to_s3(&s3_instance, &file_content, &file_uuid, &group).await;

  match file_to_s3 {
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
    _ => {}
  }

  // Add datasource to Postgres
  let datasource_s3_id: String = format!(
    "s3://{}/{}/{}",
    &s3_instance.bucket_name, &group, &file_uuid
  );
  let pool: Pool<Postgres> = state.pool;
  let datasource_to_postgres: Result<PgRow, Error> = add_datasource_to_postgres(
    &pool,
    &file_uuid,
    &datasource_s3_id,
    &file_name,
    &file_size,
    &group,
  )
  .await;

  match datasource_to_postgres {
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
    _ => {}
  }

  // Define job name
  let job_name: String = format!("Ingestion of {}", &file_name);

  // Add ingest job to Redis
  let ingest_job_to_redis: Result<(), redis::RedisError> =
    add_job_to_redis(&pipeline, &job_uuid, &job_name, &datasource_s3_id).await;

  match ingest_job_to_redis {
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
    _ => {}
  }

  // Add ingest job to Postgres
  let ingest_job_to_postgres: Result<PgRow, Error> =
    add_job_to_postgres(&pool, &job_uuid, &job_name, &pipeline).await;

  match ingest_job_to_postgres {
    Err(e) => return (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
    _ => {}
  }

  (StatusCode::OK, "Upload file successful.".to_string())
}
