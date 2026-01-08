-- Add up migration script here

-- todos テーブル
CREATE TABLE IF NOT EXISTS todos (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  title VARCHAR(255) NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  due_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (user_id) REFERENCES users(id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

-- todos テーブルの updated_at を自動更新するためのトリガー
CREATE TRIGGER todos_updated_at_trigger
  BEFORE UPDATE ON todos FOR EACH ROW
  EXECUTE FUNCTION set_updated_at();
