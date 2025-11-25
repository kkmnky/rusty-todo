# rusty-todo

Rust で Todo アプリケーションを一から作る個人学習プロジェクトです。コマンドラインアプリを足場に、ドメインとアダプタを分けた構成や、開発効率を高めるツールチェーン整備を自力で組み立てることを目的としています。

## 目標
- Rust での CLI 実装を通じて、タスク追加・一覧・完了・削除といった基本機能を作る
- シンプルなフロントエンド（`frontend/`）も用意し、UI からタスク確認・操作できるようにする
- ドメイン層と外部入出力（ファイル永続化など）を分離し、保守しやすい構成を体験する
- `cargo make` や docker-compose などの周辺設定を自分で用意し、開発フローを整える

## 想定する構成（順次追加予定）
- `Dockerfile`, `compose.yaml`: 開発用コンテナ環境
- `Makefile.toml`: `cargo make` で fmt/clippy/test などのタスクを集約
- `rust-toolchain.toml`: Rust バージョン固定
- `src`, `kernel`, `adapter`, `api`, `registry`, `shared`: ドメインとアダプタを意識した配置
- `frontend`, `infra`, `data`, `doc`: 補助用ディレクトリ（必要に応じて作成）

## 始め方（メモ）
1. `cargo new --bin rusty-todo` でプロジェクトを初期化する
2. `cargo fmt` / `cargo clippy` / `cargo test` を基本の検証コマンドとして維持し、`cargo make` にまとめる予定
3. CLI サブコマンド（add/list/done/delete など）と永続化レイヤーを段階的に実装する

## ライセンス
ルートの `LICENSE` を参照してください。
