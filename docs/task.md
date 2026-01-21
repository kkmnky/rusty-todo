# 開発タスクリスト

1. [x] ルート構成を分離する: `backend/` に Rust プロジェクト、`frontend/` に UI を配置
2. [x] Backend を初期化する: `backend/` で `cargo new --bin rusty-todo`、`kernel/` `adapter/` `api/` `registry/` `shared/` を用意
3. [x] 共通設定を整える: `.gitignore` `rust-toolchain.toml` `Makefile.toml`（fmt/clippy/test タスク）、`Dockerfile` `compose.yaml` 叩き台を置く
4. [x] Cargo 依存を追加する: `actix-web` `serde` `serde_json` `sqlx`(+postgres) `argon2` `jsonwebtoken` `chrono` `uuid` `config` `anyhow` `thiserror`
5. [x] ドメインを定義する: User（id/name/email/password_hash/created_at/updated_at）、Todo（id/user_id/title/status/due?/created_at/updated_at）、Status(enum)
6. [x] DB 基盤を整える: 接続設定（.env/config）、接続プール、`sqlx migrate` 初期化、ヘルスチェックエンドポイント
7. [ ] ユーザ CRUD を実装する: ドメイン/ユースケース/リポジトリ/エンドポイント（例: `POST /auth/signup`, `POST /auth/login`, `GET/PUT/DELETE /users/{id}` 等）
   - エンドポイント（/api/v1 配下、rusty-book-manager と同一仕様）:
     | メソッド | パス | 説明 | 関数名 |
     | --- | --- | --- | --- |
     | POST | `/api/v1/users` | ユーザ追加 | `register_user` |
     | GET | `/api/v1/users` | ユーザ一覧取得 | `list_users` |
     | DELETE | `/api/v1/users/:user_id` | ユーザ削除 | `delete_user` |
     | GET | `/api/v1/users/me` | 自分情報取得 | `get_current_user` |
     | PUT | `/api/v1/users/me/password` | 自分パスワード更新 | `change_password` |
     | POST | `/api/v1/auth/login` | ログイン | `auth_login` |
     | POST | `/api/v1/auth/logout` | ログアウト | `auth_logout` |
  - サブタスク:
    - 方針: CRUDは操作ごとにテストを分割。順番は Adapter → API。
    - ユーザ作成:
      - [x] テスト(Adapter): ユーザ作成 正常系
        - 作成成功し返却Userのname/emailが一致する
        - usersに1件作成されpassword_hashは平文と不一致
        - password_hashのbcrypt検証がtrue
      - [x] テスト(Adapter): ユーザ作成 異常系
        - 同一emailで2回作成するとSqlExecuteErrorになる
      - [x] テスト(API): `POST /api/v1/users` 正常系
      - [x] テスト(API): `POST /api/v1/users` 異常系
        - [x] email不正でValidationErrorになる
        - [x] リポジトリ失敗でSqlExecuteErrorになる
    - ユーザ一覧取得:
      - [x] テスト(Adapter): ユーザ一覧取得 正常系
        - [x] 作成前後で件数が1件増える
        - [x] 作成ユーザが一覧に含まれname/emailが一致する
      - [x] テスト(Adapter): ユーザ一覧取得 異常系（対象なし）
      - [x] テスト(API): `GET /api/v1/users` 正常系
        - [x] 200とユーザ配列を返す
        - [x] 返却配列にname/email/idが含まれる
      - [x] テスト(API): `GET /api/v1/users` 異常系（対象なし）
    - ユーザ削除:
      - [x] テスト(Adapter): ユーザ削除 正常系
        - 削除が成功する
        - usersから対象ユーザが取得できない
      - [x] テスト(Adapter): ユーザ削除 異常系
        - 存在しないユーザIDで削除するとEntityNotFoundErrorになる
      - [x] テスト(API): `DELETE /api/v1/users/:user_id` 正常系
        - 204を返す
        - レスポンスボディが空
      - [x] テスト(API): `DELETE /api/v1/users/:user_id` 異常系
        - 不正なuser_idで400を返す
        - 存在しないuser_idで404を返す
    - 認証:
      - 方針:
        - ログインはメール+パスワードで認証
        - メールでユーザ取得 → パスワード検証
        - JWTを発行し有効期限は1時間
        - JWTはHS256 + JWT_SECRETで署名
        - アクセストークンはRedisに保存（token -> user_id, TTL=1h）
        - ログアウトはトークン削除
        - 認証情報はAuthRepositoryで扱い、Userとは分離する
        - ログインレスポンスはaccessToken/ expiresIn/ userIdを返す
      - [x] テスト(Adapter): 認証情報取得（メール）正常系
        - メール指定で認証情報が取得できる
        - 取得したemail/idが一致する
        - password_hashが取得できる
      - [ ] テスト(Adapter): 認証情報取得（メール）異常系
        - 存在しないメールならNoneを返す
      - [ ] テスト(Adapter): トークン保存（Redis）正常系
        - アクセストークンが保存される
        - TTLが1時間で設定される
      - [ ] テスト(Adapter): トークン削除（Redis）正常系
        - アクセストークンが削除される
      - [ ] テスト(API): `POST /api/v1/auth/login` 正常系
        - アクセストークンを返す
        - 期限情報を返す
      - [ ] テスト(API): `POST /api/v1/auth/login` 異常系
        - パスワード不一致で401を返す
        - 存在しないメールで401を返す
      - [ ] テスト(API): `POST /api/v1/auth/logout` 正常系
        - アクセストークンが削除される
      - [ ] テスト(API): `POST /api/v1/auth/logout` 異常系
    - 自分情報取得:
      - [x] テスト(Adapter): ユーザ取得（ID）正常系
        - ID指定でユーザが取得できる
        - 取得したname/email/idが一致する
      - [x] テスト(Adapter): ユーザ取得（ID）異常系
        - 存在しないIDならNoneを返す
      - [ ] テスト(API): `GET /api/v1/users/me` 正常系
      - [ ] テスト(API): `GET /api/v1/users/me` 異常系
    - パスワード更新:
      - [ ] テスト(Adapter): パスワード更新 正常系
      - [ ] テスト(Adapter): パスワード更新 異常系
      - [ ] テスト(API): `PUT /api/v1/users/me/password` 正常系
      - [ ] テスト(API): `PUT /api/v1/users/me/password` 異常系
8. [ ] ユーザ用マイグレーションを作成・適用する: users テーブル、必要ならインデックス
9. [ ] ユーザ機能の動作確認をする: 統合テストまたは手動でサインアップ→ログイン→取得/更新/削除を確認
10. [ ] Todo CRUD を実装する: ドメイン/ユースケース/リポジトリ/エンドポイント（`GET /todos`, `GET /todos/{id}`, `POST /todos`, `PUT /todos/{id}`, `DELETE /todos/{id}`）
    - エンドポイント（/api/v1 配下、book API を Todo に読み替え）:
      - GET `/todos`
      - POST `/todos`
      - GET `/todos/:todo_id`
      - PUT `/todos/:todo_id`
      - DELETE `/todos/:todo_id`
      - GET `/todos/completed`（完了済み一覧。books/checkouts の一覧相当の補助ビュー）
      - POST `/todos/:todo_id/complete`（完了アクション。books/:id/checkouts 相当）
      - PUT `/todos/:todo_id/complete/:completion_id/reopen`（再オープン。returned 相当）
      - GET `/todos/:todo_id/history`（状態遷移履歴。checkout-history 相当）
    - サブタスク:
      - [ ] ドメイン/ユースケースを定義（作成・取得・更新・削除・完了/再開・履歴）
      - [ ] リポジトリ実装（Todo 保存/検索、履歴管理）
      - [ ] ハンドラ/ルーター実装（上記エンドポイント）
      - [ ] 入力バリデーションとエラーハンドリング
11. [ ] Todo 用マイグレーションを作成・適用する: todos テーブル（user_id FK, status, timestamps 等）
12. [ ] Todo 機能の動作確認をする: 統合テストまたは手動で作成→一覧→更新→削除を確認
13. [ ] テストを揃える: ユニット（ドメイン/ハッシュ/JWT）、統合（サインアップ→ログイン→Todo CRUD）、Lint/Format（`cargo fmt`, `cargo clippy`, `cargo test`）
14. [ ] Frontend を初期化する: `frontend/` を Vite+React 等でセットアップし、eslint/prettier を設定
15. [ ] Frontend 認証を作る: サインアップ/ログイン画面、JWT 保存と付与を実装
16. [ ] Frontend Todo UI を作る: 一覧/追加/編集/削除/完了トグル、API 連携と基本バリデーション
17. [ ] ドキュメントを整える: README にセットアップ手順、環境変数例（`.env.example`）、主要コマンド、API エンドポイントを記載
18. [ ] コンテナ動作を確認する: `docker compose up` で backend+db(+frontend) が起動することを確認

## 作業記録 (2026-01-13)

- ユーザ追加の正常系テストを追加し、`register_user` の戻り値に 201 を含める形へ調整
- ユーザ追加のAPIモデル（`CreateUserRequest`/`UserResponse`）とハンドラ/ルートを追加
- kernel に `User`/`CreateUser`/`UserId` と `UserRepository` を追加
- `AppError` の `IntoResponse` を実装し、バリデーションは 400 のみ返す形に整理
- タスク7の表とテストチェックボックスを最新化

次: adapter と registry の実装
