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
                    .col(string(Messages::Content))
                    .col(string(Messages::AuthorUsername))
                    .col(integer(Messages::DirectoryId))
                    .col(timestamp_with_time_zone(Messages::CreatedAt))
                    .col(integer_null(Messages::ParentId))
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
                    .foreign_key(
                        ForeignKey::create()
                            .from(Messages::Table, Messages::ParentId)
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
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
}
