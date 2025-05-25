use super::Directory;
use super::Message;
use super::User;
use entity::{directory, messages, users};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::collections::VecDeque;

pub async fn get_users(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
    users::Entity::find().all(db).await
}

pub async fn get_directory(db: &DatabaseConnection, id: i32) -> Result<Vec<Directory>, DbErr> {
    let mut results: Vec<Directory> = Vec::new();
    let mut queue: VecDeque<i32> = VecDeque::new();

    match directory::Entity::find_by_id(id).one(db).await? {
        Some(root_node) => {
            results.push(root_node.clone());
            queue.push_back(root_node.id);

            while let Some(current_parent_id) = queue.pop_front() {
                let children = directory::Entity::find()
                    .filter(directory::Column::ParentId.eq(Some(current_parent_id)))
                    .all(db)
                    .await?;

                for child in children {
                    results.push(child.clone());
                    queue.push_back(child.id);
                }
            }
        }
        None => {
            return Err(DbErr::RecordNotFound(format!(
                "Directory with id {} not found",
                id
            )));
        }
    }

    Ok(results)
}

pub async fn get_thread(db: &DatabaseConnection, id: i32) -> Result<Vec<Message>, DbErr> {
    messages::Entity::find()
        .filter(messages::Column::DirectoryId.eq(id))
        .all(db)
        .await
}
