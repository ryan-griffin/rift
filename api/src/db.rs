use crate::entity::{
	directory, directory::Model as Directory, messages, messages::Model as Message, users,
	users::Model as User,
};
use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use std::collections::VecDeque;

pub async fn get_users(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
	users::Entity::find().all(db).await
}

pub async fn get_user(db: &DatabaseConnection, username: &str) -> Result<User, DbErr> {
	users::Entity::find()
		.filter(users::Column::Username.eq(username))
		.one(db)
		.await?
		.ok_or(DbErr::RecordNotFound(format!(
			"User with username {username} not found"
		)))
}

pub async fn create_user(db: &DatabaseConnection, user: User) -> Result<User, DbErr> {
	users::ActiveModel {
		username: Set(user.username),
		name: Set(user.name),
		password: Set(user.password),
	}
	.insert(db)
	.await
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
				"Directory with id {id} not found"
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

pub async fn get_message(db: &DatabaseConnection, id: i32) -> Result<Message, DbErr> {
	messages::Entity::find()
		.filter(messages::Column::Id.eq(id))
		.one(db)
		.await?
		.ok_or(DbErr::RecordNotFound(format!(
			"Message with id {id} not found"
		)))
}

pub async fn create_message(
	db: &DatabaseConnection,
	author_username: String,
	message: Message,
) -> Result<Message, DbErr> {
	messages::ActiveModel {
		author_username: Set(author_username),
		content: Set(message.content),
		directory_id: Set(message.directory_id),
		parent_id: Set(message.parent_id),
		created_at: Set(Utc::now().into()),
		..Default::default()
	}
	.insert(db)
	.await
}
