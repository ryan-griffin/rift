use crate::m1_create_users_table::Users;
use crate::m2_create_directory_table::Directory;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Messages::Table)
					.if_not_exists()
					.col(pk_auto(Messages::Id))
					.col(string_len(Messages::Content, 4000))
					.col(string(Messages::AuthorUsername))
					.col(integer(Messages::DirectoryId))
					.col(timestamp_with_time_zone(Messages::CreatedAt))
					.col(timestamp_with_time_zone_null(Messages::EditedAt))
					.col(integer_null(Messages::ParentId))
					.col(timestamp_with_time_zone_null(Messages::DeletedAt))
					// Live messages must have content; tombstones are exempt
					// because deletion blanks it.
					.check(Expr::cust(
						r#"deleted_at IS NOT NULL OR char_length("content") > 0"#,
					))
					.foreign_key(
						ForeignKey::create()
							.from(Messages::Table, Messages::AuthorUsername)
							.to(Users::Table, Users::Username)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.from(Messages::Table, Messages::DirectoryId)
							.to(Directory::Table, Directory::Id)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					// Deletion means tombstoning via deleted_at, never
					// physical deletes — so this FK has no ON DELETE action,
					// and deletes that would orphan replies are rejected.
					.index(
						Index::create()
							.name("idx_messages_id_directory_id")
							.col(Messages::Id)
							.col(Messages::DirectoryId)
							.unique(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_messages_parent_same_thread")
							.from(Messages::Table, (Messages::ParentId, Messages::DirectoryId))
							.to(Messages::Table, (Messages::Id, Messages::DirectoryId))
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		// Hot read path: thread views filter by directory_id.
		// A plain index cannot be declared inline in CREATE TABLE.
		manager
			.create_index(
				Index::create()
					.name("idx_messages_directory_id_created_at")
					.table(Messages::Table)
					.col(Messages::DirectoryId)
					.col(Messages::CreatedAt)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Messages::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Messages {
	Table,
	Id,
	Content,
	AuthorUsername,
	DirectoryId,
	CreatedAt,
	ParentId,
	DeletedAt,
	EditedAt,
}
