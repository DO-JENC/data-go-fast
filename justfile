set dotenv-load := true
TEST_PASS := "password123"
TEST_USER_EMAIL := "alice@example.com"
DATABASE_URL := "postgres://" + env_var("POSTGRES_USER") + ":" + env_var("POSTGRES_PASSWORD") + "@localhost:" + env_var("DATABASE_PORT") + "/" + env_var("POSTGRES_USER")

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
    cd front && pnpm run dev

dev-server:
    cargo watch -C server -x run

dev-worker:
    cargo watch -C worker -x run

# Run development environment
dev: up
    #!/usr/bin/env bash
    set -e
    trap 'kill 0' EXIT
    just dev-front &
    just dev-server &
    just dev-worker &
    wait


# Stop external services (redis, S3, postgres) and remove volumes, networks, and orphaned containers
clean:
    docker compose down --volumes --remove-orphans
    @echo "Environment fully wiped (containers, volumes, and orphans removed)."


_init: _doctor
    @if [ ! -f .env ]; then \
        cp .env.example .env; \
        echo ".env created from .env.example."; \
        printf "Press enter to continue or Ctrl+C to edit .env manually..."; \
        read _ < /dev/tty; \
    fi

_wait-for-db:
    #!/usr/bin/env bash
    echo "Waiting for PostgreSQL schema..."
    for i in $(seq 1 60); do
        docker compose exec -T db psql -U ${POSTGRES_USER} -d ${POSTGRES_USER} \
            -c "SELECT 1 FROM groups LIMIT 1" > /dev/null 2>&1 \
            && echo "PostgreSQL is ready!" && exit 0
        echo "  attempt $i/60..."
        sleep 1
    done
    echo "PostgreSQL schema did not initialize in time." && exit 1

_seed:
    #!/usr/bin/env bash
    echo "Seeding database..."
    psql {{DATABASE_URL}} <<EOF
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
	@command -v pnpm >/dev/null 2>&1 || (echo "pnpm is missing. Run: corepack enable pnpm"; exit 1)
	@command -v psql >/dev/null 2>&1 || (echo "psql is missing. Install PostgreSQL client tools (e.g., 'sudo apt install postgresql-client')."; exit 1)
	@echo "All systems go!"
