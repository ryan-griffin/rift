use super::Directory;
use super::User;
use entity::{directory, users};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

pub async fn get_users(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
    users::Entity::find().all(db).await
}

pub async fn get_directory(db: &DatabaseConnection) -> Result<Vec<Directory>, DbErr> {
    directory::Entity::find().all(db).await
}
