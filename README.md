# README

探索Rust Axum + SeaORM + Postgres 开发web的实践

## install tools

```bash
cargo install sea-orm-cli
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

## Start the DB

```bash
sh ./scripts/init_db.sh
```

## Init Migration

```bash
sea-orm-cli migrate up
```

## dev

```bash

cargo watch -q -c -w crates/services/web-server/src/ -w crates/libs/ -w .cargo/ -x "run -p web-server"

cargo watch -q -c -w crates/services/ -x "run --example quick_dev"

```
