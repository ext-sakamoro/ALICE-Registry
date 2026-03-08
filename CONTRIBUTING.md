# Contributing to ALICE-Registry

## Prerequisites

- Rust 1.70+
- Node.js 22+ (フロントエンド)
- Docker / Docker Compose (統合テスト)

## Build

```bash
cd services/api-gateway && cargo build
cd services/core-engine && cargo build
```

## Test

```bash
cd services/api-gateway && cargo test
cd services/core-engine && cargo test
```

## Lint

```bash
cd services/api-gateway && cargo clippy -- -W clippy::all -W clippy::pedantic
cd services/core-engine && cargo clippy -- -W clippy::all -W clippy::pedantic
cargo fmt -- --check
```

## Code Style

- `cargo fmt` — 必須
- `cargo clippy -- -W clippy::all -W clippy::pedantic` — 警告ゼロ
- コメント・コミットメッセージは日本語
- 作成者: Moroya Sakamoto

## Architecture

- `services/api-gateway` — JWT/API-Key認証、レートリミット、リバースプロキシ
- `services/core-engine` — OCI Push/Pull、レイヤー差分、コンテンツ検索
- `frontend/` — Next.js 15 ダッシュボード
