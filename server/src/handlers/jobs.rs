use redis::{ Client, aio::MultiplexedConnection, JsonAsyncCommands, RedisResult};
use uuid::Uuid;
use serde_json::{json, Value};

pub async fn add_job_to_redis(pipeline: Value, job_uuid: Uuid) -> RedisResult<()> {

    let redis_connection_string: String = std::env::var("REDIS_CONNECTION_STRING").expect("REDIS_CONNECTION_STRING environment variable not found.");

    let client: Client = redis::Client::open(redis_connection_string)?;
    let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    let redis_json: Value = json!({
        "name": "filename",
        "status": "pending",
        "datasource_id": "datasource_s3_id",
        "pipeline": pipeline
    });

    let response: () = con.json_set(job_uuid.to_string(), "$", &redis_json).await?;
    Ok(response)

}
