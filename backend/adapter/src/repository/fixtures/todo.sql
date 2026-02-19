INSERT INTO users(id, name, email, password_hash)
VALUES
  ('f0f6de0a-8e7f-4ca3-a0ed-2db4e8d51056', 'Todo Other User', 'todo-other-user@example.com', '$2b$12$busePsuArd81KUB45zH1su2EC4Jz4PMGHvwQO5vpUMdfhC7JwmSJm');

INSERT INTO todos (id, user_id, title, completed, due_at, created_at, updated_at)
VALUES
  ('10f0d6f2-c464-4f4c-92f0-6d87f7324f11', '75ef7d75-3b57-4f54-8e8e-fdb65738690c', 'target-old', false, NULL, '2026-01-01 09:00:00+00', '2026-01-01 09:00:00+00'),
  ('67d4895c-b538-4c81-846d-c3f08d41ecbe', '75ef7d75-3b57-4f54-8e8e-fdb65738690c', 'target-new', false, NULL, '2026-01-01 10:00:00+00', '2026-01-01 10:00:00+00'),
  ('550db8cb-458c-43d4-a581-78cd2920cbd3', 'f0f6de0a-8e7f-4ca3-a0ed-2db4e8d51056', 'other-old', false, NULL, '2026-01-01 08:00:00+00', '2026-01-01 08:00:00+00'),
  ('7f5d6b8e-3b58-43e0-b595-7a9e058d2eed', 'f0f6de0a-8e7f-4ca3-a0ed-2db4e8d51056', 'other-new', false, NULL, '2026-01-01 11:00:00+00', '2026-01-01 11:00:00+00');
