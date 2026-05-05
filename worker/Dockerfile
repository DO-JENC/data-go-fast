FROM lukemathwalker/cargo-chef@sha256:00c3c07c51d092325df88f0df2d626cd4302e12933f179ba154509cc314d6c2a AS chef
WORKDIR /app

# Build recipe file that contains project skeleton (including all Cargo.toml paths)
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies using Docker cache
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin worker

# Run the app
FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN useradd -m app
USER app

COPY --from=builder /app/target/release/worker /usr/local/bin
ENTRYPOINT ["/usr/local/bin/worker"]
