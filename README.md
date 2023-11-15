# README

探索Rust Axum + SeaORM + Postgres 开发web的实践

## install tools

```bash
cargo install sea-orm-cli
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

## Start the DB and migration

```bash
sh ./scripts/init_db.sh

cd ./crates/libs/ && sea-orm-cli migrate up
```

## dev

```bash

cargo watch -q -c -w crates/services/web-server/src/ -w crates/libs/ -w .cargo/ -x "run -p web-server"

cargo watch -q -c -w crates/services/ -x "run --example quick_dev"

```

## About Migration

SeaORM 中常用的一些操作以及 migration 中的一些使用：

```bash
# 初始化生成migration directory
sea-orm-cli migrate init
# 创建migrations
sea-orm-cli migrate generate create_xxx
# 执行所有未执行的migrate操作, 可以通过 -n number制定执行的数量
sea-orm-cli migrate up
# 回退最近的一次migratem, 同样可以通过-n number制定执行的数量
sea-orm-cli migrate down
# 删除所有的表，并重新执行所有的migrate
sea-orm-cli migrate fresh
# 回退所有执行的migrate 并重新执行所有的migrate
sea-orm-cli migrate refresh
```

通过数据库表信息生成entity

```bash
sea-orm-cli generate entity \
-o ./crates/libs/lib-core/src/model/entity
```

在migration 中初始化表的数据和删除数据的例子：

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
  let db = manager.get_connection();
  let transaction = db.begin().await?;
  let u: users::ActiveModel = users::ActiveModel {
   username: Set("demo1".to_string()),
   password: Set("".to_string()),
   ..Default::default()
  }
  .insert(&transaction)
  .await?
  .into();

  let pwd = pwd::hash_pwd(&ContentToHash {
   content: "welcome".to_string(),
   salt: u.password_salt.clone().unwrap(),
  })
  .unwrap();

  users::ActiveModel {
   password: Set(pwd),
   ..u
  }
  .update(&transaction)
  .await?;

  transaction.commit().await?;
  Ok(())
 }

 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
  let db = manager.get_connection();

  Entity::delete_many()
   .filter(users::Column::Username.eq("demo1"))
   .exec(db)
   .await?;

  Ok(())
 }
}
```

在migration中修改表给表添加新的字段，创建外键，删除表字段，删除外键的使用例子：

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
 async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
  manager
   .alter_table(
    sea_query::Table::alter()
     .table(Tasks::Table)
     .add_column(
      &mut ColumnDef::new(Alias::new("project_id"))
       .uuid()
       .not_null(),
     )
     .to_owned(),
   )
   .await?;
  manager
   .create_foreign_key(
    sea_query::ForeignKey::create()
     .name("project_id")
     .from(Tasks::Table, Tasks::ProjectId)
     .to(Projects::Table, Projects::Id)
     .on_delete(ForeignKeyAction::Cascade)
     .to_owned(),
   )
   .await?;
  Ok(())
 }

 async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
  manager
   .drop_foreign_key(
    sea_query::ForeignKey::drop()
     .name("project_id")
     .table(Tasks::Table)
     .to_owned(),
   )
   .await?;
  manager
   .alter_table(
    sea_query::Table::alter()
     .table(Tasks::Table)
     .drop_column(Alias::new("project_id"))
     .to_owned(),
   )
   .await?;
  Ok(())
 }
}
```