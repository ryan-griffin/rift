use crate::m1_create_users_table::Users;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(AuthSessions::Table)
					.if_not_exists()
					.col(string_len(AuthSessions::TokenHash, 64).primary_key())
					.col(string_len(AuthSessions::Username, 32))
					.col(timestamp_with_time_zone(AuthSessions::ExpiresAt))
					.foreign_key(
						ForeignKey::create()
							.from(AuthSessions::Table, AuthSessions::Username)
							.to(Users::Table, Users::Username)
							.on_update(ForeignKeyAction::Cascade)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_auth_sessions_username")
					.table(AuthSessions::Table)
					.col(AuthSessions::Username)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_auth_sessions_expires_at")
					.table(AuthSessions::Table)
					.col(AuthSessions::ExpiresAt)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(AuthSessions::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum AuthSessions {
	Table,
	TokenHash,
	Username,
	ExpiresAt,
}
