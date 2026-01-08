# データモデル

```mermaid
erDiagram
    USERS ||--o{ TODOS : has

    USERS {
        uuid id PK
        varchar name
        varchar email
        varchar password_hash
        timestamptz created_at
        timestamptz updated_at
    }

    TODOS {
        uuid id PK
        uuid user_id FK
        varchar title
        boolean completed
        timestamptz due_at
        timestamptz created_at
        timestamptz updated_at
    }
```

補足:
- nullable: `todos.due_at`
- unique: `users.email`
