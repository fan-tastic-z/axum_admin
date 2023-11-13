pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20231025_080739_create_tasks_table;
mod m20231113_111449_init_users_table_data;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20231025_080739_create_tasks_table::Migration),
            Box::new(m20231113_111449_init_users_table_data::Migration),
        ]
    }
}
