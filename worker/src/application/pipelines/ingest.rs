// use common::infra::s3::config::S3Instance;
// use common::queue::models::Job;
// use s3::bucket::Bucket;
// use sqlx::Row;
// use sqlx::types::Json;
// use sqlx::{Pool, Postgres, postgres::PgRow, query};
// use uuid::Uuid;

// async fn get_document(job: Job, s3_instance: S3Instance) -> Result<String, String> {
//   let mut bucket: Box<Bucket> = Bucket::new(
//     &s3_instance.bucket_name,
//     s3_instance.region.clone(),
//     s3_instance.credentials.clone(),
//   )
//   .unwrap();
//   bucket.set_path_style();

//   let file_name: String = format!(
//     "{}.{:?}",
//     job.datasource_id.trim_start_matches("s3://data-go-fast/"),
//     job.pipeline.r#type
//   );
//   match bucket.get_object(file_name).await {
//     Ok(response) => Ok(response.to_string().map_err(|e| e.to_string())?),
//     Err(error) => return Err(error.to_string()),
//   }
// }

// pub async fn ingest_json(
//   job: Job,
//   s3_instance: S3Instance,
//   pool: Pool<Postgres>,
// ) -> Result<PgRow, String> {
//   let request: PgRow = query("SELECT id FROM datasources WHERE s3_id = $1")
//     .bind(job.datasource_id.clone())
//     .fetch_one(&pool)
//     .await
//     .map_err(|e| e.to_string())?;
//   let job_id: Uuid = request.get("id");
//   let document = get_document(job, s3_instance).await?;

//   match query(
//     "
//         INSERT INTO
//         json_table (datasource_id, document)
//         VALUES ($1, $2) RETURNING *;
//     ",
//   )
//   .bind(job_id)
//   .bind(Json(document))
//   .fetch_one(&pool)
//   .await
//   {
//     Ok(response) => Ok(response),
//     Err(e) => return Err(e.to_string()),
//   }
// }
