set dotenv-load := true 
TEST_PASS := "password123"
TEST_USER_EMAIL := "alice@example.com"

default: 
    @just --list

# Initialise the project environments
init: 
    @if [ ! -f .env ]; then \
    cp .env.example .env; \
    echo ".env created from .env.example."; \
    read -p "Press enter to continue or Ctrl+C to edit .env manually..."; \
    fi

# Build and start the containers
up: init
    docker-compose up -d

down: 
    docker-compose down

seed:
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