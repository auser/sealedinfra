-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  disabled BOOLEAN DEFAULT FALSE,
  admin BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (username)
);

CREATE UNIQUE INDEX IF NOT EXISTS users_username_idx ON users (username);

CREATE TABLE IF NOT EXISTS emails (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
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

CREATE TABLE IF NOT EXISTS user_verifications (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  hash TEXT NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users (id),
  UNIQUE (hash)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_verifications_hash_idx ON user_verifications (hash);
CREATE INDEX IF NOT EXISTS user_verifications_user_id_idx ON user_verifications (user_id);

DO
$$
BEGIN
  CREATE TYPE repo_visibility AS ENUM ('public', 'private');
EXCEPTION
  WHEN duplicate_object THEN
    RAISE NOTICE 'type repo_visibility already exists';
END
$$;

CREATE TABLE IF NOT EXISTS repositories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  owner INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT DEFAULT '',
  visibility repo_visibility NOT NULL,
  license TEXT DEFAULT '',
  forked_from INTEGER,
  mirrored_from TEXT,
  archived BOOLEAN DEFAULT FALSE,
  disabled BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (owner) REFERENCES users (id)
);

do
$$
    BEGIN
        CREATE TYPE ssh_key_type AS ENUM (
            'ssh-rsa',
            'ecdsa-sha2-nistp256',
            'ecdsa-sha2-nistp384',
            'ecdsa-sha2-nistp521',
            'ssh-ed25519'
            );
    exception
        WHEN duplicate_object THEN
            RAISE NOTICE 'type ssh_key_type already exists';
    end
$$;

CREATE TABLE IF NOT EXISTS ssh_keys
(
    id          serial
        constraint ssh_keys_pk
            primary key,
    owner       integer                                not null
        constraint ssh_keys_users_id_fk
            references users
            on delete cascade,
    title       varchar(64)                            not null,
    fingerprint char(47)                               not null,
    algorithm   ssh_key_type                           not null,
    key         bytea                                  not null,
    created_at  timestamp with time zone default now() not null,
    expires_at  timestamp with time zone
);

create unique index if not exists ssh_keys_fingerprint_uindex
    on ssh_keys (fingerprint);

create unique index if not exists ssh_keys_key_uindex
    on ssh_keys (key);
