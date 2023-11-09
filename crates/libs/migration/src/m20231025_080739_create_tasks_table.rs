use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Tasks::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Tasks::Id)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()")
							.primary_key(),
					)
					.col(ColumnDef::new(Tasks::Title).string().not_null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Tasks::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum Tasks {
	Table,
	Id,
	Title,
}