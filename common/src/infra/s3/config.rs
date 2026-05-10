use awscreds::Credentials;
use s3::region::Region;

#[derive(Clone)]
pub struct S3Instance {
  pub endpoint: String,
  pub bucket_name: String,

  pub access_key: String,
  pub secret_access_key: String,

  pub region: Region,
  pub credentials: Credentials,
}

pub fn init_s3_instance() -> S3Instance {
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
    access_key: Some(access_key.clone()),
    secret_key: Some(secret_access_key.clone()),
    security_token: None,
    session_token: None,
    expiration: None,
  };

  // Initiate S3Instance
  S3Instance {
    endpoint: endpoint,
    bucket_name: bucket_name,

    access_key: access_key,
    secret_access_key: secret_access_key,

    region: region,
    credentials: credentials,
  }
}
