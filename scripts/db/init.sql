-- Provides functions to generate UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE datasource_type AS ENUM ('csv', 'json');
CREATE TYPE job_status AS ENUM ('pending', 'running', 'done', 'error');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    hash_password TEXT NOT NULL UNIQUE
);

CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE user_groups (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID REFERENCES groups(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, group_id)
);

CREATE TABLE datasources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    s3_id TEXT NOT NULL,
    name TEXT NOT NULL,
    file_type datasource_type NOT NULL,
    size FLOAT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    group_id UUID REFERENCES groups(id),
    UNIQUE(name, group_id)
);

CREATE TABLE json_table (
    id SERIAL PRIMARY KEY,
    datasource_id UUID UNIQUE REFERENCES datasources(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    pipeline JSONB NOT NULL,
    status job_status DEFAULT 'pending',
    result_datasource_id UUID REFERENCES datasources(id)
);


CREATE TABLE job_datasources (
    job_id UUID REFERENCES jobs(id) ON DELETE CASCADE,
    datasource_id UUID REFERENCES datasources(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, datasource_id)
);

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
