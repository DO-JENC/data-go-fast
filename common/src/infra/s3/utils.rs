use s3::Bucket;
use uuid::Uuid;

use crate::infra::s3::config::S3Instance;

// Download raw bytes from S3 (format: "s3://bucket/group-uuid/file-uuid.ext")
pub async fn download_from_s3(s3: &S3Instance, s3_id: &str) -> Result<Vec<u8>, String> {
  let (group, file, ext) = parse_s3_id(s3_id)?;
  let key = format!("/{}/{}.{}", group, file, ext);

  let mut bucket: Box<Bucket> =
    Bucket::new(&s3.bucket_name, s3.region.clone(), s3.credentials.clone())
      .map_err(|e| format!("Failed to create S3 bucket: {:?}", e))?;

  bucket.set_path_style();
  let response = bucket
    .get_object(key)
    .await
    .map_err(|e| format!("S3 download failed: {}", e))?;

  Ok(response.bytes().to_vec())
}

// Upload filtered bytes to S3 with a new UUID
pub async fn upload_to_s3(
  s3: &S3Instance,
  content: &[u8],
  group: &Uuid,
  ext: &str,
) -> Result<String, String> {
  let new_file_uuid = Uuid::new_v4();
  let key = format!("/{}/{}.{}", group, new_file_uuid, ext);

  let mut bucket: Box<Bucket> =
    Bucket::new(&s3.bucket_name, s3.region.clone(), s3.credentials.clone())
      .map_err(|e| format!("Failed to create S3 bucket: {:?}", e))?;

  bucket.set_path_style();
  bucket
    .put_object(&key, content)
    .await
    .map_err(|e| format!("S3 upload failed: {}", e))?;

  Ok(format!(
    "s3://{}/{}/{}.{}",
    s3.bucket_name, group, new_file_uuid, ext
  ))
}

pub fn parse_s3_id(s3_id: &str) -> Result<(Uuid, Uuid, String), String> {
  let parts: Vec<&str> = s3_id.split('/').collect();
  if parts.len() < 4 {
    return Err(format!("Invalid s3_id format: {}", s3_id));
  }

  let group = Uuid::parse_str(parts[parts.len() - 2])
    .map_err(|e| format!("Invalid group UUID in s3_id: {}", e))?;

  let last = parts[parts.len() - 1];
  let (file_str, ext) = last.split_once('.').unwrap_or((last, "csv"));
  let file = Uuid::parse_str(file_str).map_err(|e| format!("Invalid file UUID in s3_id: {}", e))?;

  Ok((group, file, ext.to_string()))
}
