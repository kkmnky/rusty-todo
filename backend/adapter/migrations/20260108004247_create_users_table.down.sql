-- Add down migration script here
DROP TRIGGER IF EXISTS users_updated_at_trigger ON users;
DROP TABLE IF EXISTS users;
