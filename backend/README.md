# backend

Rust 製の API/CLI サービス。本番用を見据え、レイヤード構成でドメインと外部 I/O を分離する。

## フォルダ構成
| パス | 役割 |
| --- | --- |
| `src/bin/` | エントリポイント（現状は axum で Hello World を返す） |
| `kernel/` | ドメインモデルとユースケースの中心 |
| `api/` | HTTP/CLI の公開インターフェース層。ハンドラや DTO を配置予定 |
| `adapter/` | DB・キャッシュなど外部システムへの具象実装（リポジトリやクライアント） |
| `registry/` | 依存関係の組み立て。DI 相当の配線や設定の注入 |
| `shared/` | 環境判定などクロスカッティングなユーティリティ |

## レイヤードアーキテクチャ

参考にしているrusty-book-manager同様レイヤードアーキテクチャを採用する。

| 名前 | 概要 |
| --- | --- |
| `api/` | 画面からの入力情報を受け取るレイヤー。axumのルーティング周りを描いている |
| `kernel/` | 受け取った入力情報をアプリケーションが扱いやすいデータ形式に変換しつつ、必要な処理をかける |
| `adapter/` | 永続化層（今回だとPostgresqlとRedis）にアクセスし、データを保存するレイヤー |

レイヤーのルールとして以下3つをおく。

1. 上位レイヤーは、同一レイヤー内、もしくは下位レイヤーのコンポーネントを呼び出して利用する
2. 隣接するレイヤーのコンポーネントしか呼び出さない
3. 下位レイヤーは上位レイヤーのコンポーネントを呼び出さない

## 命名方針
- API/Usecase は Register を基本にする
- Adapter/Repository/イベントは Create を基本にする

## ローカル実行
- `cargo fmt` / `cargo clippy` / `cargo test` を基本の検証コマンド。
- `cargo run --bin app` で開発用 HTTP サーバー起動（ポート 8080、`ENV` でログレベル切り替え）。
- `compose.yaml` で Postgres・Redis と合わせて起動可能（`.env` に各種ポート/認証を設定）。本番向け設定は今後追加。

## テスト方針
- adapter は `sqlx::test` を基本にする。
- api は `rstest` を基本にする。
- api の単体テストは分岐/バリデーション/エラーハンドリングに集中する。
- `axum::Json` の型不正/必須項目不足などフレームワーク既定挙動（400/422）は、単体テストではなく API 統合テストで最小1ケースのみ確認する。
- api の統合テストは最小限のスモークテストに絞る。
- 統合テストは `backend/api/tests/integration/main.rs` 配下で管理する。

## オブザーバビリティ
- 基本
  - OpenTelemetryをベースとする
  - Rustでは相性が良い`tracing`クレートを使ってトレースやログを出力する
  - オブザーバビリティバックエンドはGrafanaスタックを使う
    - トレースは Grafana Tempo（OTLP/gRPC, `localhost:4317`）。
    - 設定ファイルなどは `/observability` に配置し、makers コマンドで独立起動する。
  - Resource は `service.name=rusty-todo`、`deployment.environment=dev|prod` を付与する。
- リクエストトレース
  - axum 向けに `tower_http::trace::TraceLayer` で 1 リクエスト 1 span とする。
  - `tracing-actix-web` は `actix-web` 専用のため不採用。
  - `SetRequestIdLayer` / `PropagateRequestIdLayer` を併用して request_id を扱う。
  - span 名（`otel.name`）は `{METHOD} {path}`（例: `GET /api/v1/users/:user_id`）とする。
  - `path` は `MatchedPath`（ルートテンプレート）を優先し、取得できない場合のみ実パスを使う。
  - span フィールドは `request_id` / `method` / `path` を含める。
  - レスポンス時に `status` / `latency_ms` を event 出力する。
  - 認証後は `user_id` を span に `record` する。
  - `X-Request-Id` 優先、無ければ UUID 生成。
- DB/Redis 計装
  - DB は `sqlx-tracing` を本採用する。
  - 通常クエリは `sqlx-tracing` の `Pool` 経由で計装する。
  - トランザクションは SQLx 純正（`sqlx::Transaction`）で管理し、トランザクション内クエリは `db.query` span を明示的に付与する。
  - Redis は `otel-instrumentation-redis` を本採用する。
  - DB/Redis は通常時もトレース対象とし、機密情報とバインド値は出さない。
  - DB のバインド値は dev/prod ともに出力しない。
  - Redis は `command`（`GET`/`SET`/`DEL` など）のみ出力し、key/value は出力しない。
- ログ出力
  - `tracing-subscriber` を使う。
  - レベルは dev=`debug`, prod=`info`（`RUST_LOG` 優先）。
  - ただし `opentelemetry_sdk` / `opentelemetry` は `info` 以上に固定し、`BatchSpanProcessor.ExportingDueToTimer` などの内部 DEBUG ノイズは抑制する。
  - 形式は dev=`pretty`, prod=`json`（`LOG_FORMAT=json|pretty` で切替）。
  - サンプリングは dev/prod ともに 100% とする（学習用途のため）。
  - HTTPアクセスログのイベントレベルは以下とする。
    - `request.received`: `debug`
    - `request.completed`: `info`
    - 4xx 応答: `warn`
    - 5xx 応答: `error`
  - エラー方針は 4xx=`warn`, 5xx=`error`、`error.kind` / `error.message` / `error.cause_chain` を出す。
  - `error.kind` は `AppError` の enum 名をそのまま使う（例: `ValidationError`, `SqlExecuteError`）。
- JSONスキーマ（OpenTelemetry準拠）
  - 全イベント共通の必須キーは以下とする。
    - `timestamp`
    - `severity_text`
    - `severity_number`
    - `body`
    - `trace_id`（取得不可時は `null`）
    - `span_id`（取得不可時は `null`）
    - `resource.service.name`
    - `resource.deployment.environment`
    - `scope.name`
    - `attributes.event.name`
  - `observed_timestamp` と `trace_flags` は当面省略する。
- 機密
  - `password` / `token` は出力しない。
  - `email` は平文で出力しない。
  - `attributes.user.email_masked` に `a***@example.com` 形式で出力する。
  - マスクルールはローカル部の先頭1文字のみ残し、以降は `*` とする。
