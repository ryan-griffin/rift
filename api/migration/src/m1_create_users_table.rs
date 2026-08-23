use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.if_not_exists()
					.col(string_len(Users::Username, 32).primary_key())
					.col(string_len(Users::Name, 64))
					.col(string(Users::Password))
					.col(timestamp_with_time_zone_null(Users::DeletedAt))
					.check(Expr::cust(r#"char_length("name") > 0"#))
					.check(Expr::cust(r#""username" ~ '^[a-z0-9_]+$'"#))
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Users::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Users {
	Table,
	Username,
	Name,
	Password,
	DeletedAt,
}
