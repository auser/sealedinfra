-- Drop existing types if they exist
DROP TYPE IF EXISTS user_role;
DROP TYPE IF EXISTS repo_visibility;
DROP TYPE IF EXISTS ssh_key_type;
DROP TYPE IF EXISTS app_status;
DROP TYPE IF EXISTS deployment_status;

-- Create ENUM types
CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE TYPE repo_visibility AS ENUM ('public', 'private');
CREATE TYPE ssh_key_type AS ENUM (
  'ssh-rsa',
  'ecdsa-sha2-nistp256',
  'ecdsa-sha2-nistp384',
  'ecdsa-sha2-nistp521',
  'ssh-ed25519'
);
CREATE TYPE app_status AS ENUM ('draft', 'published', 'archived');
CREATE TYPE deployment_status AS ENUM ('pending', 'in_progress', 'successful', 'failed');

-- Create users table
CREATE TABLE IF NOT EXISTS users (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  disabled BOOLEAN DEFAULT FALSE,
  admin BOOLEAN DEFAULT FALSE,
  role user_role DEFAULT 'user',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS users_username_idx ON users (username);

-- Create emails table
CREATE TABLE IF NOT EXISTS emails (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  user_id BIGINT NOT NULL,
  email TEXT NOT NULL,
  is_primary BOOLEAN DEFAULT FALSE,
  is_commit BOOLEAN DEFAULT FALSE,
  is_notification BOOLEAN DEFAULT FALSE,
  is_public BOOLEAN DEFAULT FALSE,
  is_verified BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  verified_at TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users (id),
  UNIQUE (email)
);

CREATE UNIQUE INDEX IF NOT EXISTS emails_user_id_idx ON emails (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS emails_email_idx ON emails (email);

-- Create user_verifications table
CREATE TABLE IF NOT EXISTS user_verifications (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  user_id BIGINT NOT NULL,
  hash TEXT NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users (id),
  UNIQUE (hash)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_verifications_hash_idx ON user_verifications (hash);
CREATE INDEX IF NOT EXISTS user_verifications_user_id_idx ON user_verifications (user_id);

-- Create repositories table
CREATE TABLE IF NOT EXISTS repositories (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  owner BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT DEFAULT '',
  visibility repo_visibility NOT NULL,
  license TEXT DEFAULT '',
  forked_from BIGINT,
  mirrored_from TEXT,
  archived BOOLEAN DEFAULT FALSE,
  disabled BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS repositories_owner_idx ON repositories (owner);
CREATE UNIQUE INDEX IF NOT EXISTS repositories_owner_name_idx ON repositories (owner, name);

-- Create ssh_keys table
CREATE TABLE IF NOT EXISTS ssh_keys (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  owner BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  fingerprint CHAR(47) NOT NULL,
  algorithm ssh_key_type NOT NULL,
  key BYTEA NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() NOT NULL,
  expires_at TIMESTAMP WITH TIME ZONE
);

CREATE UNIQUE INDEX IF NOT EXISTS ssh_keys_fingerprint_idx ON ssh_keys (fingerprint);
CREATE UNIQUE INDEX IF NOT EXISTS ssh_keys_key_idx ON ssh_keys (key);

-- Create apps table
CREATE TABLE IF NOT EXISTS apps (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name TEXT NOT NULL,
  description TEXT DEFAULT '',
  owner_id BIGINT NOT NULL REFERENCES users (id),
  status app_status DEFAULT 'draft',
  repository_id BIGINT REFERENCES repositories (id),
  app_config JSONB DEFAULT '{}',
  repository_url TEXT DEFAULT '',
  branch TEXT DEFAULT '',
  image TEXT DEFAULT '',
  tag TEXT DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (owner_id,repository_id)
);

CREATE INDEX IF NOT EXISTS apps_owner_id_idx ON apps (owner_id);
CREATE INDEX IF NOT EXISTS apps_repository_id_idx ON apps (repository_id);

-- Create collaborators table
CREATE TABLE IF NOT EXISTS collaborators (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  repository_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  permission TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (repository_id, user_id)
);

CREATE INDEX IF NOT EXISTS collaborators_repository_id_idx ON collaborators (repository_id);
CREATE INDEX IF NOT EXISTS collaborators_user_id_idx ON collaborators (user_id);

-- Create deployments table
CREATE TABLE IF NOT EXISTS deployments (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  app_id BIGINT NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
  version TEXT NOT NULL,
  status deployment_status DEFAULT 'pending',
  deployed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  deployed_by BIGINT REFERENCES users(id),
  log TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS deployments_app_id_idx ON deployments (app_id);
CREATE INDEX IF NOT EXISTS deployments_status_idx ON deployments (status);

-- Create app_logs table
CREATE TABLE IF NOT EXISTS app_logs (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  app_id BIGINT NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
  log_level TEXT NOT NULL,
  message TEXT NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS app_logs_app_id_idx ON app_logs (app_id);
CREATE INDEX IF NOT EXISTS app_logs_timestamp_idx ON app_logs (timestamp);

-- Create app_metrics table
CREATE TABLE IF NOT EXISTS app_metrics (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  app_id BIGINT NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
  cpu_usage FLOAT,
  memory_usage FLOAT,
  network_in BIGINT,
  network_out BIGINT,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS app_metrics_app_id_idx ON app_metrics (app_id);
CREATE INDEX IF NOT EXISTS app_metrics_timestamp_idx ON app_metrics (timestamp);