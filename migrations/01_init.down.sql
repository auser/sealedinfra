-- Revert migration: Drop all tables and types

-- Drop tables
DROP TABLE IF EXISTS app_metrics;
DROP TABLE IF EXISTS app_logs;
DROP TABLE IF EXISTS deployments;
DROP TABLE IF EXISTS collaborators;
DROP TABLE IF EXISTS apps;
DROP TABLE IF EXISTS ssh_keys;
DROP TABLE IF EXISTS repositories;
DROP TABLE IF EXISTS user_verifications;
DROP TABLE IF EXISTS emails;
DROP TABLE IF EXISTS users;

-- Drop types
DROP TYPE IF EXISTS deployment_status;
DROP TYPE IF EXISTS app_status;
DROP TYPE IF EXISTS ssh_key_type;
DROP TYPE IF EXISTS repo_visibility;
DROP TYPE IF EXISTS user_role;

-- Drop any remaining indexes (if not automatically dropped with tables)
DROP INDEX IF EXISTS app_metrics_timestamp_idx;
DROP INDEX IF EXISTS app_metrics_app_id_idx;
DROP INDEX IF EXISTS app_logs_timestamp_idx;
DROP INDEX IF EXISTS app_logs_app_id_idx;
DROP INDEX IF EXISTS deployments_status_idx;
DROP INDEX IF EXISTS deployments_app_id_idx;
DROP INDEX IF EXISTS collaborators_user_id_idx;
DROP INDEX IF EXISTS collaborators_repository_id_idx;
DROP INDEX IF EXISTS apps_repository_id_idx;
DROP INDEX IF EXISTS apps_owner_id_idx;
DROP INDEX IF EXISTS ssh_keys_key_idx;
DROP INDEX IF EXISTS ssh_keys_fingerprint_idx;
DROP INDEX IF EXISTS repositories_owner_name_idx;
DROP INDEX IF EXISTS repositories_owner_idx;
DROP INDEX IF EXISTS user_verifications_user_id_idx;
DROP INDEX IF EXISTS user_verifications_hash_idx;
DROP INDEX IF EXISTS emails_email_idx;
DROP INDEX IF EXISTS emails_user_id_idx;
DROP INDEX IF EXISTS users_username_idx;