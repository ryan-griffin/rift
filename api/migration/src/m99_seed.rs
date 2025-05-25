use crate::m1_create_users_table::Users;
use crate::m2_create_directory_table::Directory;
use crate::m3_create_messages_table::Messages;
use sea_orm_migration::{prelude::*, sea_orm::sqlx::types::chrono};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let users_insert = Query::insert()
            .into_table(Users::Table)
            .columns([Users::Username, Users::Name])
            .values_from_panic(vec![
                ["alice".into(), "Alice".into()],
                ["bob".into(), "Bob".into()],
                ["joe".into(), "Joe".into()],
            ])
            .to_owned();
        manager.exec_stmt(users_insert).await?;

        let root_insert = Query::insert()
            .into_table(Directory::Table)
            .columns([Directory::Name, Directory::Type, Directory::ParentId])
            .values_from_panic(vec![[
                "Root".into(),
                "folder".into(),
                Option::<i32>::None.into(),
            ]])
            .to_owned();
        manager.exec_stmt(root_insert).await?;

        let level1_insert = Query::insert()
            .into_table(Directory::Table)
            .columns([Directory::Name, Directory::Type, Directory::ParentId])
            .values_from_panic(vec![
                ["General".into(), "thread".into(), Some(1).into()],
                ["Gaming".into(), "folder".into(), Some(1).into()],
                ["Programming".into(), "folder".into(), Some(1).into()],
                ["Announcements".into(), "thread".into(), Some(1).into()],
                ["Memes".into(), "thread".into(), Some(1).into()],
                ["Events".into(), "folder".into(), Some(1).into()],
                ["Help".into(), "thread".into(), Some(1).into()],
            ])
            .to_owned();
        manager.exec_stmt(level1_insert).await?;

        let level2_insert = Query::insert()
            .into_table(Directory::Table)
            .columns([Directory::Name, Directory::Type, Directory::ParentId])
            .values_from_panic(vec![
                ["Roblox".into(), "thread".into(), Some(3).into()],
                ["Fortnite".into(), "thread".into(), Some(3).into()],
                ["Minecraft".into(), "folder".into(), Some(3).into()],
                ["TypeScript".into(), "thread".into(), Some(4).into()],
                ["Rust".into(), "thread".into(), Some(4).into()],
            ])
            .to_owned();
        manager.exec_stmt(level2_insert).await?;

        let level3_insert = Query::insert()
            .into_table(Directory::Table)
            .columns([Directory::Name, Directory::Type, Directory::ParentId])
            .values_from_panic(vec![
                ["Mods".into(), "thread".into(), Some(11).into()],
                ["Modpacks".into(), "thread".into(), Some(11).into()],
            ])
            .to_owned();
        manager.exec_stmt(level3_insert).await?;

        let messages_insert = Query::insert()
            .into_table(Messages::Table)
            .columns([
                Messages::Content,
                Messages::AuthorUsername,
                Messages::DirectoryId,
                Messages::CreatedAt,
                Messages::ParentId,
            ])
            .values_from_panic(vec![
                [
                    "hey everyone! welcome to the general chat 👋".into(),
                    "alice".into(),
                    2.into(),
                    chrono::Utc::now().into(),
                    Option::<i32>::None.into(),
                ],
                [
                    "hello chat".into(),
                    "bob".into(),
                    2.into(),
                    chrono::Utc::now().into(),
                    Option::<i32>::None.into(),
                ],
                [
                    "hi".into(),
                    "bob".into(),
                    2.into(),
                    chrono::Utc::now().into(),
                    Some(2).into(),
                ],
                [
                    "hello everyone! welcome to the roblox chat 👋".into(),
                    "bob".into(),
                    9.into(),
                    chrono::Utc::now().into(),
                    Option::<i32>::None.into(),
                ],
                [
                    "yes".into(),
                    "alice".into(),
                    9.into(),
                    chrono::Utc::now().into(),
                    Option::<i32>::None.into(),
                ],
                [
                    "hello".into(),
                    "joe".into(),
                    9.into(),
                    chrono::Utc::now().into(),
                    Option::<i32>::None.into(),
                ],
            ])
            .to_owned();
        manager.exec_stmt(messages_insert).await?;

        Ok(())
    }
}
