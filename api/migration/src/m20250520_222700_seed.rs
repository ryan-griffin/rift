use crate::m20250520_213905_create_users_table::Users;
use crate::m20250520_213906_create_directory_table::Directory;
use sea_orm_migration::prelude::*;

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

        Ok(())
    }
}
