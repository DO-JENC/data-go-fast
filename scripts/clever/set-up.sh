#!/usr/bin/env bash
set -e

# ==============================================================================
# CONFIGURATION
# ==============================================================================
ORG_ID="orga_e4d64185-94d8-4d10-9d26-31b39dafd743"
BACKEND_ALIAS="backend"
FRONT_ALIAS="front"

BACKEND_APP_NAME="data-go-fast-backend"
FRONT_APP_NAME="data-go-fast-front"

DB_ADDON_NAME="data-go-fast-db"
REDIS_ADDON_NAME="data-go-fast-redis"
S3_ADDON_NAME="data-go-fast-s3"

JWT_SECRET_VAL="your_secure_secret"

# ==============================================================================
# 1. APPLICATION INITIALIZATION & SCALING
# ==============================================================================
echo "Creating Clever Cloud applications..."
clever create --type rust "$BACKEND_APP_NAME" --alias "$BACKEND_ALIAS" --org "$ORG_ID"
clever create --type docker "$FRONT_APP_NAME" --alias "$FRONT_ALIAS" --org "$ORG_ID"

echo "Scaling up backend instance for Rust build workloads..."
clever scale --flavor S --alias "$BACKEND_ALIAS"

# ==============================================================================
# 2. RUNTIME ENVIRONMENT CONFIGURATION
# ==============================================================================
echo "Configuring operational parameters..."
clever env set CC_RUST_BIN "server" --alias "$BACKEND_ALIAS"
clever env set CC_RUN_COMMAND "./target/release/server" --alias "$BACKEND_ALIAS"
clever env set SERVER_PORT "8080" --alias "$BACKEND_ALIAS"
clever env set CC_WORKER_COMMAND "./target/release/worker" --alias "$BACKEND_ALIAS"
clever env set CC_DOCKERFILE "front/Dockerfile" --alias "$FRONT_ALIAS"
clever env set JWT_SECRET "$JWT_SECRET_VAL" --alias "$BACKEND_ALIAS"

# ==============================================================================
# 3. SERVICE ADD-ONS PROVISIONING & DATA-LINKING
# ==============================================================================
echo "Linking database infrastructure..."
clever service link-addon "$DB_ADDON_NAME" --alias "$BACKEND_ALIAS"
eval "$(clever env --alias "$BACKEND_ALIAS" --format shell)"
DATABASE_URL="postgres://${POSTGRESQL_ADDON_USER}:${POSTGRESQL_ADDON_PASSWORD}@${POSTGRESQL_ADDON_HOST}:${POSTGRESQL_ADDON_PORT}/${POSTGRESQL_ADDON_DB}"
clever env set DATABASE_URL "$DATABASE_URL" --alias "$BACKEND_ALIAS"

echo "Linking cache & message queue..."
clever service link-addon "$REDIS_ADDON_NAME" --alias "$BACKEND_ALIAS"
eval "$(clever env --alias "$BACKEND_ALIAS" --format shell)"
REDIS_CONNECTION_STRING="${REDIS_URL}"
clever env set REDIS_CONNECTION_STRING "$REDIS_CONNECTION_STRING" --alias "$BACKEND_ALIAS"

echo "Linking S3 object storage..."
clever service link-addon "$S3_ADDON_NAME" --alias "$BACKEND_ALIAS"
eval "$(clever env --alias "$BACKEND_ALIAS" --format shell)"
S3_ENDPOINT="https://${CELLAR_ADDON_HOST}"
clever env set S3_ENDPOINT "$S3_ENDPOINT" --alias "$BACKEND_ALIAS"
clever env set AWS_ACCESS_KEY_ID "${CELLAR_ADDON_KEY_ID}" --alias "$BACKEND_ALIAS"
clever env set AWS_SECRET_ACCESS_KEY "${CELLAR_ADDON_KEY_SECRET}" --alias "$BACKEND_ALIAS"
clever env set AWS_DEFAULT_REGION "garage" --alias "$BACKEND_ALIAS"
clever env set BUCKET_NAME "data-go-fast" --alias "$BACKEND_ALIAS"

# ==============================================================================
# 4. CROSS-APPLICATION DOMAIN EXTRACTION
# ==============================================================================
echo "Resolving cluster topology for frontend..."
BACKEND_DOMAIN=$(clever domain --alias "$BACKEND_ALIAS" | head -n 1 | tr -d '[:space:]')

if [ -z "$BACKEND_DOMAIN" ]; then
  echo "Critical failure: Cloud routing table returned empty domain identifier for alias '$BACKEND_ALIAS'."
  exit 1
fi

# Sanitize trailing slashes from the retrieved endpoint
BACKEND_DOMAIN="${BACKEND_DOMAIN%/}"
BACKEND_URL="https://${BACKEND_DOMAIN}"
echo "Identified Backend Entrypoint: $BACKEND_URL"

clever env set BACKEND_URL "$BACKEND_URL" --alias "$FRONT_ALIAS"

# ==============================================================================
# 5. EXECUTE PRODUCTION CODE DEPLOYMENTS
# ==============================================================================
echo "Triggering production compilation hooks..."
clever deploy --alias "$BACKEND_ALIAS"
clever deploy --alias "$FRONT_ALIAS"

echo "System integration workflow completed successfully."
