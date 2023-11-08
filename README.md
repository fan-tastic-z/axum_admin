# README

该项目用于一点一点探索Rust Axum + SeaORM + Postgres 开发web的实践

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
