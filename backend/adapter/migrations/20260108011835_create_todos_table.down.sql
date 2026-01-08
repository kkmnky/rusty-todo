-- Add down migration script here
DROP TRIGGER IF EXISTS todos_updated_at_trigger ON todos;
DROP TABLE IF EXISTS todos;
