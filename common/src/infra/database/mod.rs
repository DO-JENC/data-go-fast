pub mod config;
pub mod datasource;
pub mod job;

pub use sqlx::{Pool, Postgres};
pub type PgPool = Pool<Postgres>;
