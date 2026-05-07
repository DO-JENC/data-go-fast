use common::infra::database::datasource::Datasource;
use sqlx::PgPool;
use uuid::Uuid;

use awscreds::Credentials;
use axum::{Json, body::Bytes, extract::{Multipart, Path, State}, http::StatusCode, response::IntoResponse};
use redis::RedisResult;
use s3::{Bucket, error::S3Error, region::Region, request::ResponseData};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::handlers::jobs::add_job_to_redis;

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

async fn add_file_to_s3(
  file_content: Bytes,
  file_uuid: Uuid,
  group: &str,
) -> Result<ResponseData, S3Error> {
  // Load S3 related environment variables
  let region: String = std::env::var("AWS_DEFAULT_REGION")
    .expect("AWS_DEFAULT_REGION environment variable not found.");
  let endpoint: String =
    std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT environment variable not found.");
  let access_key: String =
    std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID environment variable not found.");
  let secret_access_key: String = std::env::var("AWS_SECRET_ACCESS_KEY")
    .expect("AWS_SECRET_ACCESS_KEY environment variable not found.");
  let bucket_name: String =
    std::env::var("BUCKET_NAME").expect("BUCKET_NAME environment variable not found.");

  // Set up S3 objects
  let region: Region = Region::Custom {
    region: region.to_owned(),
    endpoint: endpoint.to_owned(),
  };

  let credentials: Credentials = Credentials {
    access_key: Some(access_key),
    secret_key: Some(secret_access_key),
    security_token: None,
    session_token: None,
    expiration: None,
  };

  let mut bucket: Box<Bucket> =
    Bucket::new(&bucket_name, region.clone(), credentials.clone()).unwrap();

  // Add file to S3 bucket
  bucket.set_path_style();
  bucket
    .put_object(format!("/{}/{}.csv", group, file_uuid), &file_content)
    .await
}

pub async fn add_ingest_job_to_redis(metadata: Value, job_uuid: Uuid) -> RedisResult<()> {
  // Extract file metadata from request
  let file_type: &Value = metadata.get("type").unwrap();
  let header: &Value = metadata.get("header").unwrap();

  let pipeline: Value = json!([{
      "op": "ingest",
      "type": file_type,
      "header": header,
  }]);

  // Add ingest job to Redis
  add_job_to_redis(pipeline, job_uuid).await
}

pub async fn csv_ingestion_handler(mut multipart: Multipart) -> impl IntoResponse {
  // Handles body request
  let mut file: Option<Bytes> = None;
  let mut metadata: Option<Value> = None;

  while let Some(field) = multipart.next_field().await.unwrap() {
    let key = field.name().unwrap_or("").to_string();

    match key.as_str() {
      "file" => {
        let data = field.bytes().await.unwrap();
        file = Some(data);
      }
      "metadata" => {
        let text = field.text().await.unwrap();
        metadata = Some(serde_json::from_str(&text).unwrap());
      }
      _ => {}
    }
  }

  // TO-DO : Implement authentication and change group dynamically
  let group: &str = "ADMIN";
  let file_uuid: Uuid = Uuid::new_v4();
  let job_uuid: Uuid = Uuid::new_v4();

  println!("File UUID: {}", file_uuid);
  println!("Job Ingestion UUID: {}", job_uuid);

  // Upload file to S3 Bucket
  let file: Bytes = file.unwrap();
  let file_to_s3: Result<ResponseData, S3Error> = add_file_to_s3(file, file_uuid, group).await;

  match file_to_s3 {
    Ok(response) => {
      println!("S3 upload successful: {:?}", response);
    }
    Err(e) => {
      eprintln!("S3 upload failed: {:?}", e);
      return (StatusCode::BAD_REQUEST, format!("Erreur S3 : {:?}", e));
    }
  }

  // Add ingest job to Redis
  let metadata: Value = metadata.unwrap();
  let ingest_job_to_redis: Result<(), redis::RedisError> =
    add_ingest_job_to_redis(metadata, job_uuid).await;

  match ingest_job_to_redis {
    Ok(response) => {
      println!("Ingest job successfully added to Redis: {:?}", response);
    }
    Err(e) => {
      eprintln!("Error adding ingest job to Redis: {:?}", e);
      return (StatusCode::BAD_REQUEST, format!("Erreur Redis : {:?}", e));
    }
  }

  match ingest_job_to_redis {
    Ok(_) => (StatusCode::OK, "Upload file successful.".to_string()),
    Err(e) => (StatusCode::BAD_REQUEST, format!("Error: {:?}", e)),
  }
}
