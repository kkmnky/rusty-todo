# rusty-todo

Rust で Todo アプリケーションを一から作る個人学習プロジェクトです。コマンドラインアプリを足場に、ドメインとアダプタを分けた構成や、開発効率を高めるツールチェーン整備を自力で組み立てることを目的としています。
以前勉強した[rusty-book-manager](https://github.com/rust-web-app-book/rusty-book-manager)をベースに作成しています。

## 目標
- Rust での CLI 実装を通じて、タスク追加・一覧・完了・削除といった基本機能を作る
- シンプルなフロントエンド（`frontend/`）も用意し、UI からタスク確認・操作できるようにする
- ドメイン層と外部入出力（ファイル永続化など）を分離し、保守しやすい構成を体験する
- `cargo make` や docker-compose などの周辺設定を自分で用意し、開発フローを整える

## 構成

### フロントエンド
- `frontend/`: UI 実装用の枠のみ。まだ未作成で、ビルド/実行手順は後日追記予定。

### バックエンド
- `backend/`: Rust 製のコアアプリ。主なサブディレクトリは `src`（エントリ）、`kernel`（ドメイン）、`adapter`（外部 I/O）、`api`（公開インタフェース）、`registry`（組み立て）、`shared`（共通ユーティリティ）。
- `backend/Makefile.toml`, `backend/rust-toolchain.toml`: fmt/clippy/test をまとめたタスクとツールチェーン固定。
- `backend/compose.yaml`: ローカル開発用コンテナ起動。
- 主な使用ライブラリ: axum（HTTP）, tokio（非同期）, tracing/tracing-subscriber（ログ）, strum（環境判定）。
- 詳細なアーキテクチャは [backend/README.md](/backend/README.md) を参照。

### データベース
- `backend/compose.yaml` で `postgres:15`（DB）と `redis:alpine`（キャッシュ）を起動する想定。
- スキーマ定義やマイグレーション手順はこれから整備する。

## ライセンス
ルートの `LICENSE` を参照してください。
