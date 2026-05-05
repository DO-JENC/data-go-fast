use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool_from_env() -> Result<PgPool, sqlx::Error> {
  let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  PgPoolOptions::new()
    .max_connections(10)
    .min_connections(1)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&database_url)
    .await
}

#[cfg(test)]
mod tests {
  use super::*; // import everything from the parent module (everything above)

  #[tokio::test] // mark this function as a test and tokio allow to run it asynchronously
  async fn test_create_pool_from_env() {
    dotenvy::dotenv().ok(); // load environment variables from .env file (dotenv crate is deprecated)

    let pool = create_pool_from_env().await; // create the pool from the environment variables
    assert!(pool.is_ok()); // verify that the Result is Ok

    let unwrapped_pool = pool.unwrap(); // unwrap the Result to get the PgPool
    assert!(!unwrapped_pool.is_closed()); // verify that the connection is open

    let ping_result = sqlx::query("SELECT 1").fetch_one(&unwrapped_pool).await;
    assert!(ping_result.is_ok()); // verify that the ping query is successful

    let row = ping_result.unwrap(); // unwrap the Result to get the value
    let value: i32 = row.get(0); // get the value from the row
    assert_eq!(value, 1); // verify that the ping query returns 1
  }
}
