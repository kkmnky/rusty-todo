# 開発タスクリスト

1. [x] ルート構成を分離する: `backend/` に Rust プロジェクト、`frontend/` に UI を配置
2. [x] Backend を初期化する: `backend/` で `cargo new --bin rusty-todo`、`kernel/` `adapter/` `api/` `registry/` `shared/` を用意
3. [x] 共通設定を整える: `.gitignore` `rust-toolchain.toml` `Makefile.toml`（fmt/clippy/test タスク）、`Dockerfile` `compose.yaml` 叩き台を置く
4. [x] Cargo 依存を追加する: `actix-web` `serde` `serde_json` `sqlx`(+postgres) `argon2` `jsonwebtoken` `chrono` `uuid` `config` `anyhow` `thiserror`
5. [x] ドメインを定義する: User（id/name/email/password_hash/created_at/updated_at）、Todo（id/user_id/title/status/due?/created_at/updated_at）、Status(enum)
6. [x] DB 基盤を整える: 接続設定（.env/config）、接続プール、`sqlx migrate` 初期化、ヘルスチェックエンドポイント
7. [x] ユーザ CRUD を実装する: ドメイン/ユースケース/リポジトリ/エンドポイント（例: `POST /auth/signup`, `POST /auth/login`, `GET/PUT/DELETE /users/{id}` 等）
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
    - ユーザAPIの認証追加（JWT検証のみ）
      - [x] テスト(API): `GET /api/v1/users` 認証必須
        - [x] Authorizationヘッダがないと401を返す
        - [x] 不正JWTで401を返す
        - [x] Authorizationヘッダありで200を返す
      - [x] テスト(API): `DELETE /api/v1/users/:user_id` 認証必須
        - [x] Authorizationヘッダがないと401を返す
        - [x] Authorizationヘッダありで204を返す
    - 認証:
      - 方針:
        - ログインはメール+パスワードで認証
        - メールでユーザ取得 → パスワード検証
        - JWTを発行し有効期限は1時間（現状は簡易トークン。後でJWT実装に切替）
        - JWTはHS256 + AUTH_JWT_SECRETで署名
        - アクセストークンはRedisに保存（token -> user_id, TTL=1h）
        - ログアウトはトークン削除
        - 認証情報はAuthRepositoryで扱い、Userとは分離する
        - ログインレスポンスはaccessToken/ expiresIn/ userIdを返す
      - [x] テスト(Adapter): 認証情報取得（メール）正常系
        - メール指定で認証情報が取得できる
        - 取得したemail/idが一致する
        - password_hashが取得できる
      - [x] テスト(Adapter): 認証情報取得（メール）異常系
        - 存在しないメールならNoneを返す
      - [x] テスト(Adapter): トークン保存（Redis）正常系
        - アクセストークンが保存される
        - TTLが1時間で設定される
      - [x] テスト(Adapter): トークン削除（Redis）正常系
        - アクセストークンが削除される
      - [x] テスト(API): `POST /api/v1/auth/login` 正常系
        - アクセストークンを返す
        - 期限情報を返す
      - [x] ログインをJWTに切り替え
        - [x] JWT生成と検証のユニットテストを追加する
          - [x] 署名検証が成功する
          - [x] sub が user_id と一致する
          - [x] 不正な署名で検証に失敗する
        - [x] JWT検証エラーの詳細をログに残す
      - [x] テスト(API): `POST /api/v1/auth/login` 異常系
        - [x] パスワード不一致で401を返す
        - [x] 存在しないメールで401を返す
      - [x] テスト(API): `POST /api/v1/auth/logout` 正常系
        - [x] Authorizationヘッダのアクセストークンが削除される
        - [x] 204を返す
      - [x] テスト(API): `POST /api/v1/auth/logout` 異常系
        - [x] Authorizationヘッダがないと401を返す
        - [x] 無効なアクセストークンで401を返す
    - 自分情報取得:
      - [x] テスト(Adapter): ユーザ取得（ID）正常系
        - ID指定でユーザが取得できる
        - 取得したname/email/idが一致する
      - [x] テスト(Adapter): ユーザ取得（ID）異常系
        - 存在しないIDならNoneを返す
      - [x] テスト(API): `GET /api/v1/users/me` 正常系
        - [x] AuthorizationヘッダのJWTが検証される
        - [x] JWTのsubで取得したユーザのid/name/emailを返す
        - [x] 200を返す
      - [x] テスト(API): `GET /api/v1/users/me` 異常系
        - [x] Authorizationヘッダがないと401を返す
        - [x] 不正JWTで401を返す
        - [x] JWTは有効だがユーザが存在しない場合は404を返す
    - パスワード更新:
      - [x] テスト(Adapter): パスワード更新 正常系
        - [x] password_hashが更新される
        - [x] 新パスワードの検証がtrueになる
      - [x] テスト(Adapter): パスワード更新 異常系
        - [x] 存在しないuser_idでEntityNotFoundErrorになる
        - [x] 現在パスワード不一致でUnauthorizedになる
      - [x] テスト(API): `PUT /api/v1/users/me/password` 正常系
        - [x] Authorizationヘッダ付きで204を返す
      - [x] テスト(API): `PUT /api/v1/users/me/password` 異常系
        - [x] Authorizationヘッダがないと401を返す
        - [x] 不正JWTで401を返す
        - [x] 現在パスワード不一致で401を返す
        - [x] バリデーションエラーで400を返す
7.5. [x] リファクタリング
   - [x] テスト方針を整理して段階的に移行する
     - [x] Adapter は `sqlx::test` を基本にする
     - [x] API は `rstest` を基本にする
     - [x] API の単体テストは分岐/バリデーション/エラーハンドリングに集中する
     - [x] API の統合テストは最小限のスモークテストに絞る
   - [x] ユーザのユースケース化は認証のユースケース実装が完了してから着手
     - [x] ユーザ登録のUsecase化（Usecase追加・API移行・回帰確認）
       - [x] kernel: `usecase/user/register.rs` を追加（Input/Output定義、`UserRepository::create` 呼び出し）
       - [x] kernel: `usecase/user/mod.rs` と `usecase/mod.rs` を更新（モジュール公開）
       - [x] api: `register_user` をUsecase経由に切り替え（入力変換はハンドラ内で実施）
       - [x] テスト: 既存APIテストで回帰確認（必要ならUsecaseの最小ユニットテスト追加）
     - [x] ユーザ一覧のUsecase化（Usecase追加・API移行・回帰確認）
     - [x] ユーザ削除のUsecase化（Usecase追加・API移行・回帰確認）
     - [x] 自分情報取得のUsecase化（Usecase追加・API移行・未存在はEntityNotFound）
     - [x] パスワード更新のUsecase化（Usecase追加・API移行・回帰確認）
   - [x] ログ出力の整備（共通のログ方針/出力の確認）
     - 方針詳細は `backend/README.md` の「ログ/トレース方針」を参照
     - 実装タスク（リクエスト/ログ基盤）
       - [x] `backend/src/bin/app.rs` に `SetRequestIdLayer` / `PropagateRequestIdLayer` / `TraceLayer` を組み込み、1リクエスト1spanを有効化
       - [x] `X-Request-Id` 優先、未指定時 UUID 生成のルールを実装
       - [x] `request.received`=`debug`、`request.completed`=`info`、4xx=`warn`、5xx=`error` のイベント出力を実装
       - [x] リクエストspanに `request_id` / `method` / `path`、レスポンスイベントに `status` / `latency_ms` を出力
       - [x] 認証後に `user_id` を span へ record する処理を実装
     - 実装タスク（エラー/機密）
       - [x] `error.kind` を `AppError` enum名で出力する共通処理を実装
       - [x] `error.message` / `error.cause_chain` を構造化して出力する
       - [x] `attributes.user.email_masked`（先頭1文字+`*`）のマスク処理を実装
     - 実装タスク（DB/Redis計装）
       - [x] `sqlx-tracing` を組み込み、DBスパンをトレースへ出力
       - [x] `otel-instrumentation-redis` を組み込み、Redisスパンをトレースへ出力
       - [x] Redisログは `command` のみ出力し、key/value は非出力
     - 実装タスク（ノイズログ抑制）
       - [x] `BatchSpanProcessor.ExportingDueToTimer`（`opentelemetry_sdk` 内部 DEBUG ログ）が出力されないようにログフィルタを調整する
8. [x] ユーザ用マイグレーションを作成・適用する: users テーブル、必要ならインデックス
9. [x] ユーザ機能の動作確認をする: 統合テストまたは手動でサインアップ→ログイン→取得/更新/削除を確認
10. [ ] Frontend「My Todos」の最低限ユースケースを実装する
    - 目的:
      - ログイン中ユーザが自分のTodoを日次管理できる最小機能に絞る
    - 最低限ユースケース:
      - [ ] Todoを追加する
        - 入力はタイトル必須のみ（まずは最小）
        - 追加成功後に一覧へ即時反映する
        - テスト（Adapter → API）:
          - [x] テスト(Adapter): Todo追加 正常系
            - [x] 作成成功し返却Todoの title/assignee_user_id/completed/due_at が一致する
            - [x] todos に1件作成される
          - [x] テスト(Adapter): Todo追加 異常系
            - [x] 存在しない assignee_user_id で作成すると SqlExecuteError になる
          - [x] テスト(API): `POST /api/v1/todos` 正常系
          - [x] テスト(API): `POST /api/v1/todos` 異常系
            - [x] title不正でValidationErrorになる
            - [x] Authorizationヘッダがないと401を返す
            - [x] 不正JWTで401を返す
            - [x] リポジトリ失敗でSqlExecuteErrorになる
      - [ ] 自分のTodo一覧を表示する
        - 初期表示でTodo一覧を取得して表示する
        - 0件時は空状態メッセージを表示する
      - [ ] Todoの完了/未完了を切り替える
        - 一覧上で1操作で状態更新できる
      - [ ] Todoタイトルを編集する
        - 既存タイトルを更新し、一覧へ反映する
      - [ ] Todoを削除する
        - 誤操作防止の確認後に削除し、一覧から除外する
    - 画面状態（最低限）:
      - [ ] 読み込み中表示
      - [ ] 更新失敗時のエラー表示
      - [ ] 空状態表示（Todoなし）
