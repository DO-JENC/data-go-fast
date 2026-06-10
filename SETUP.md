# Setup & Usage

## Prerequisites

- **Rust** (latest stable)
- **Docker** + **Docker Compose** : for PostgreSQL, Redis, and Garage S3
- **pnpm** : enable via `corepack enable pnpm`
- **psql** : PostgreSQL client (`sudo apt install postgresql-client`)
- **cargo-watch** (optional, for dev) : `cargo install cargo-watch`

## Quickstart

### With `just`

```bash
# Start infrastructure + seed DB + launch all services
just dev
```

This runs `docker compose up -d`, waits for PostgreSQL, seeds the database
(with a test user `alice@example.com` / `password123`), then starts the server,
worker, and frontend in parallel.

To stop:

```bash
just down        # stop containers (data persists)
just clean       # stop + delete volumes (full wipe)
```

### Without `just`

```bash
# Setup environment
cp .env.example .env

# Start infrastructure
docker compose up -d

# Seed a test user (tables & sample data are auto-created by PostgreSQL init scripts)
export $(grep -v '^#' .env | xargs)
DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${DATABASE_PORT}/${POSTGRES_USER}"

psql "$DATABASE_URL" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
psql "$DATABASE_URL" -c "
  INSERT INTO users (email, hash_password)
  SELECT 'alice@example.com', crypt('password123', gen_salt('bf', 8))
  WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'alice@example.com');"

# 4. Run the server (terminal 1)
cargo run -p server

# 5. Run the worker (terminal 2)
cargo run -p worker

# 6. Run the frontend (terminal 3)
cd front && pnpm install && pnpm run dev
```

Services listen on:
- Server: `localhost:3000`
- Worker: `http://localhost:3001`
- Frontend: `http://localhost:5173`


## API Endpoints

### Health

```bash
curl localhost:3000/health
# → 200 OK
```

### Authentication

**Signup**

```bash
curl -X POST localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "<email>", "password": "<password>"}'
# → 201 { "access_token": "<jwt>", "token_type": "Bearer" }
```

**Login**

```bash
curl -X POST localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "<email>", "password": "<password>"}'
# → 200 { "access_token": "<jwt>", "token_type": "Bearer" }
```

Save the token:

```bash
TOKEN="<jwt>"
```

**Refresh token**

```bash
curl -X POST localhost:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -b "refresh_token=<cookie>"
# → 200 { "access_token": "<new_jwt>", "token_type": "Bearer" }
```

**Logout**

```bash
curl -X POST localhost:3000/auth/logout \
  -H "Authorization: Bearer $TOKEN"
# → 200
```

### Datasources

All datasource endpoints require `Authorization: Bearer <token>`.

**Upload a CSV file**

```bash
curl -X POST localhost:3000/datasources \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@<file>" \
  -F 'metadata={"type":"csv","header":true}'
# → 201 { "id": "<datasource_uuid>", "name": "test_data.csv", ... }
```

**List datasources**

```bash
curl localhost:3000/datasources \
  -H "Authorization: Bearer $TOKEN"
# → 200 [ { "id": "...", "name": "movies.csv", "file_type": "csv", ... } ]
```

**Get datasource by ID**

```bash
curl localhost:3000/datasources/<id> \
  -H "Authorization: Bearer $TOKEN"
```

**Delete datasource**

```bash
curl -X DELETE localhost:3000/datasources/<id> \
  -H "Authorization: Bearer $TOKEN"
# → 204 No Content
```

### Jobs

All job endpoints require `Authorization: Bearer <token>`.

**Create a job (pipeline execution)**

```bash
curl -X POST localhost:3000/jobs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "<job_name>",
    "datasource_id": "<datasource_uuid>",
    "pipeline": [
      { "op": "filter", "column": "<col>", "operator": ">", "value": <val> },
      { "op": "group_by", "by": "<col>", "aggregate": { "column": "<col>", "function": "<func>" } }
    ]
  }'
# → 202 { "job_id": "<job_uuid>" }
```

**List jobs**

```bash
curl localhost:3000/jobs \
  -H "Authorization: Bearer $TOKEN"
# → 200 [ { "id": "...", "name": "Movie > 4", "status": "done", ... } ]
```

**Get job by ID**

```bash
curl localhost:3000/jobs/<job_id> \
  -H "Authorization: Bearer $TOKEN"
# → 200 { "id": "...", "status": "done", "result_datasource_id": "...", ... }
```

When the job status is `done`, use `result_datasource_id` to fetch the result:

```bash
curl localhost:3000/datasources/<result_datasource_id> \
  -H "Authorization: Bearer $TOKEN"
```

**Download a result file from S3 (Garage)**

```bash
export AWS_ACCESS_KEY_ID=<key_id>
export AWS_SECRET_ACCESS_KEY=<secret_key>
export AWS_DEFAULT_REGION=garage

aws s3 cp s3://data-go-fast/<group_uuid>/<file_uuid>.csv ./<file> \
  --endpoint-url http://localhost:3900
```

## Pipeline Operations

Pipelines are defined as a JSON array of operations executed **sequentially**.
Each operation takes the output of the previous one as input.

### Filter

Keep only rows matching a condition.

```json
{
  "name": "<job_name>",
  "datasource_id": "<dt_id>",
  "pipeline": [
    {
      "op": "filter",
      "column": "<column>",
      "operator": "<operator>",
      "value": val
    }
  ]
}
```

Supported operators: `>`, `<`, `>=`, `<=`, `==`, `!=`

Comparisons are numeric when possible, case-insensitive string otherwise.

### Aggregate

Compute aggregates over entire columns (no grouping). Outputs JSON.

```json
{
  "name": "<job_name>",
  "datasource_id": "<dt_id>",
  "pipeline": [
    {
      "op": "aggregate",
      "columns": ["col1", "col2",...],
      "functions": ["func1", "func2",... ]
    }
  ]
}
```

Supported functions: `sum`, `avg`, `min`, `max`, `count`, `median`


### Group By

Group rows by a column and compute an aggregate on another column. Outputs CSV.

```json
{
  "name": "<job_name>",
  "datasource_id": "<dt_id>",
  "pipeline": [
    {
      "op": "group_by",
      "by": "<col>",
      "aggregate": {
        "column": "<col>",
        "function": "<func>"
       }
    }
  ]
}
```

Supported aggregate functions: `sum`, `avg`, `min`, `max`, `count`, `median`


### Full pipeline example

```json
{
  "name": "<job_name>",
  "datasource_id": "<dt_id>",
  "pipeline": [
    {
      "op": "filter",
      "column": "<col>",
      "operator": "<operator>",
      "value": val
    },
    { "op": "group_by",
      "by": "<col>",
      "aggregate": {
        "column": "<col>",
        "function": "<func>"
      }
    }
  ]
}

```

Result: CSV with columns `Rating,Year_avg`, one row per rating group.

## Graceful Shutdown

Both the server and the worker support graceful shutdown on `Ctrl+C` or `SIGTERM`.

- **Server**: stops accepting new connections, drains in-flight requests, then exits.
- **Worker**: stops polling the Redis queue, waits for running jobs to complete, then exits.
