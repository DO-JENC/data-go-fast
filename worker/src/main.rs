use apalis::prelude::*;
use apalis_redis::RedisStorage;
use common::queue::models::Job;
use common::queue::storage::get_queue_storage;

async fn job_treatment(job: Job) {
  println!("Traitement du job. {:?}", job)
}

#[tokio::main]
async fn main() {
  let storage: RedisStorage<Job> = get_queue_storage().await;
  // Then start the worker
  Monitor::new()
    .register(
      WorkerBuilder::new("worker")
        .concurrency(2)
        .backend(storage)
        .build_fn(job_treatment),
    )
    .run()
    .await
    .expect("Monitor failed");
}
