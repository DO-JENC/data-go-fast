# Apps

## Deploying an app

### Create the app

Aliases are for local handling of app (avoid having to type id each time)
```bash
clever create --type docker data-go-fast-server --alias server --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
clever env set JWT_SECRET some_secret --alias server
clever env set SERVER_PORT 8080 --alias server
clever create --type docker data-go-fast-worker --alias worker --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
clever create --type docker data-go-fast-front --alias front --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
clever env set BACKGROUND_URL "https://<domain-name>" --alias front
```

### Route each app to it's subfolder

By default, Clever Cloud looks at the root folder to build and run code. To deploy seperate subfolder apps from a single monorepo using Docker you need to specify a server-side environment variable (`CC_DOCKERFILE`) to tell it exactly where the specific Dockerfile lives for that app.

```bash
clever env set CC_DOCKERFILE "server/Dockerfile" --alias server
clever env set CC_DOCKERFILE "front/Dockerfile" --alias front
clever env set CC_DOCKERFILE "worker/Dockerfile" --alias worker
```

### Deploy the app

#### Scale the app container for build time

Rust is pretty ressource intensive during build so during deployment, container needs to be at least an S size.

```bash
clever scale --flavor S --alias server
```

```bash
clever deploy --alias server
clever deploy --alias front
clever deploy --alias worker
```

## Start or restart an app
```bash
clever restart --alias server
```

## Delete app
```bash
clever delete --app <app_id>
```

## Get env variables for app
```bash
clever env --alias server
```

# Addons

## List available addons
```bash
clever addon providers
```

## List created addons
```bash
clever addon list
```
## View plans and region options for an addon
```bash
clever addon providers show <addon>
```

## Delete addon
```bash
clever addon delete <addon-id>
```

## Creating postgres addon

### Create the addon
```bash
clever addon create postgresql-addon data-go-fast-db --plan dev --region par --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
```

### Service dependencies

#### Server
```bash
clever service link-addon data-go-fast-db --alias server
clever env set DATABASE_URL 'postgres://${POSTGRESQL_ADDON_USER}:${POSTGRESQL_ADDON_PASSWORD}@${POSTGRESQL_ADDON_HOST}:${POSTGRESQL_ADDON_PORT}/${POSTGRESQL_ADDON_DB}' --alias server
```

## Create the redis addon

### Create the addon
```bash
clever addon create redis-addon data-go-fast-redis --plan s_mono --region par --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
```

### Service dependencies

#### Server
```bash
clever service link-addon data-go-fast-db --alias server
clever env set REDIS_CONNECTION_STRING 'redis://default:${REDIS_ADDON_PASSWORD}@${REDIS_ADDON_HOST}:${REDIS_ADDON_PORT}/' --alias server
```

## Create the S3 addon

### Create the addon
```bash
clever addon create cellar-addon data-go-fast-s3 --plan s --region par --org orga_e4d64185-94d8-4d10-9d26-31b39dafd743
```

### Service dependencies

#### Server
```bash
clever service link-addon data-go-fast-s3 --alias server
clever env set S3_ENDPOINT 'https://${CELLAR_ADDON_HOST}' --alias server
clever env set AWS_ACCESS_KEY_ID '${CELLAR_ADDON_KEY_ID}' --alias server
clever env set AWS_SECRET_ACCESS_KEY '${CELLAR_ADDON_KEY_SECRET}' --alias server
clever env set AWS_DEFAULT_REGION "garage" --alias server
clever env set BUCKET_NAME "data-go-fast" --alias server
```
