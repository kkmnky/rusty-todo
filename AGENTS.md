# AGENTS

- 返答は簡潔な日本語で。不要な敬語や長文は避ける。
- 既存変更を壊さないこと。強制的な reset や不要なファイル削除は禁止。
- コード編集は可能な限り `apply_patch` を使い、小さな差分で示す。生成物の丸ごと貼り付けは避ける。
- 構成は `rusty-book-manager` に倣う。`src`/`kernel`/`adapter`/`api`/`registry`/`shared` に加え、`frontend/` も用意して UI から操作できるようにする。
- 基本検証コマンド: `cargo fmt` / `cargo clippy` / `cargo test`（将来 `cargo make` に集約）。実行できない場合は理由を明示。
- 外部ネットワークや追加ツール導入が必要な場合は事前に相談。Docker/compose 設定も段階的に追加する。
- 進め方はテスト駆動。まず Codex がテストを作成し、ユーザが実装して OK になったら次へ進む。テスト提示後、OK になるまでの具体的な手順も示す。
