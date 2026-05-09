-- Provides functions to generate UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE datasource_type AS ENUM ('csv', 'json');
CREATE TYPE job_status AS ENUM ('pending', 'running', 'done', 'error');
CREATE TYPE job_action AS ENUM ('ingest', 'filter', 'group_by', 'aggregate');

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
    type datasource_type NOT NULL,
    size FLOAT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    group_id UUID REFERENCES groups(id)
);

CREATE TABLE json_table (
    id SERIAL PRIMARY KEY,
    datasource_id UUID UNIQUE REFERENCES datasources(id) ON DELETE CASCADE,
    document JSONB NOT NULL
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    actions job_action[] NOT NULL,
    status job_status DEFAULT 'pending'
);


CREATE TABLE job_datasources (
    job_id UUID REFERENCES jobs(id) ON DELETE CASCADE,
    datasource_id UUID REFERENCES datasources(id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, datasource_id)
);
