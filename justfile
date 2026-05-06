set dotenv-load := true
TEST_PASS := "password123"
TEST_USER_EMAIL := "alice@example.com"

default:
    @just --list




# Build and start the containers
up: _init
    docker compose up -d
    @just _wait-for-db
    @just _seed
    @echo "Environment is running"

# Stop containers
down:
    docker compose down

# Stop containers and remove volumes, networks, and orphaned containers
clean:
    docker compose down --volumes --remove-orphans
    @echo "Environment fully wiped (containers, volumes, and orphans removed)."


_init:
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
