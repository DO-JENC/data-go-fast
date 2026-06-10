use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logging() {
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

  let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

  if log_format == "json" {
    tracing_subscriber::registry()
      .with(filter)
      .with(fmt::layer().json())
      .init();
  } else {
    tracing_subscriber::registry()
      .with(filter)
      .with(fmt::layer())
      .init();
  }
}
