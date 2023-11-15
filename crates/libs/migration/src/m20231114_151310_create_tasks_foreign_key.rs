use crate::{
	m20231025_080739_create_tasks_table::Tasks,
	m20231114_145659_create_project_table::Projects,
};

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
