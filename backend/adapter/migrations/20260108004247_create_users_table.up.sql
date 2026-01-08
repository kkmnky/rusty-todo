-- Add up migration script here
-- users テーブル
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- users テーブルの updated_at を自動更新するためのトリガー
CREATE TRIGGER users_updated_at_trigger
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE FUNCTION set_updated_at();
