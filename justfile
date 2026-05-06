set dotenv-load := true
TEST_PASS := "password123"
TEST_USER_EMAIL := "alice@example.com"

default:
    @just --list

# Start the necessary external services (redis, S3, postgres)
up: _init
    docker compose up -d
    @just _wait-for-db
    @just _seed
    @echo "Environment is running"

# Stop external services (redis, S3, postgres)
down:
    docker compose down

dev-front:
    cd frontend && pnpm run dev

dev-server:
    cargo watch -C server -x run

dev-worker:
    cargo watch -C worker -x run

# Run development environment
dev: up
    just dev-front & just dev-server & just dev-worker


# Stop external services (redis, S3, postgres) and remove volumes, networks, and orphaned containers
clean:
    docker compose down --volumes --remove-orphans
    @echo "Environment fully wiped (containers, volumes, and orphans removed)."


_init: _doctor
    @if [ ! -f .env ]; then \
    cp .env.example .env; \
    echo ".env created from .env.example."; \
    read -p "Press enter to continue or Ctrl+C to edit .env manually..."; \
    fi

_wait-for-db:
    @echo "Waiting for PostgreSQL to be ready..."
    @until docker compose exec -T db pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB:-postgres} > /dev/null 2>&1; do \
        sleep 1; \
    done
    @echo "PostgreSQL is ready!"

_seed:
    #!/usr/bin/env bash
    echo "Seeding database..."
    psql $DATABASE_URL <<EOF
    CREATE EXTENSION IF NOT EXISTS pgcrypto;

    WITH group_upsert AS (
        INSERT INTO groups (name)
        VALUES ('Test Admins')
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
    ),
    user_upsert AS (
        INSERT INTO users (email, hash_password)
        VALUES (
            '{{TEST_USER_EMAIL}}',
            crypt('{{TEST_PASS}}', gen_salt('bf', 8))
        )
        ON CONFLICT (email) DO UPDATE SET hash_password = EXCLUDED.hash_password
        RETURNING id
    )
    INSERT INTO user_groups (user_id, group_id)
    SELECT user_upsert.id, group_upsert.id
    FROM user_upsert, group_upsert
    ON CONFLICT DO NOTHING;

    \i scripts/db/insert.sql
    EOF

# Check if all necessary tools are installed
_doctor:
    @echo "Checking dependencies..."
    @command -v cargo >/dev/null 2>&1 || (echo "cargo is not installed. Install it from https://rustup.rs/"; exit 1)
    @command -v cargo-watch >/dev/null 2>&1 || (echo "cargo-watch is missing. Run: cargo install cargo-watch"; exit 1)
    @command -v docker >/dev/null 2>&1 || (echo "docker is missing. Install Docker Engine."; exit 1)
    @command -v pnpm >/dev/null 2>&1 || (echo "pnpm is missing. Run: npm install -g pnpm"; exit 1)
    @echo "All systems go!"
