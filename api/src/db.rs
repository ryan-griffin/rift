use crate::entity::{
	directory, directory::Model as Directory, messages, messages::Model as Message, users,
	users::Model as User,
};
use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use std::collections::VecDeque;

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
	users::Entity::find().all(db).await
}

pub async fn find_user_by_username(
	db: &DatabaseConnection,
	username: &str,
) -> Result<Option<User>, DbErr> {
	users::Entity::find()
		.filter(users::Column::Username.eq(username))
		.one(db)
		.await
}

pub async fn insert_user(
	db: &DatabaseConnection,
	username: String,
	name: String,
	password: String,
) -> Result<User, DbErr> {
	users::ActiveModel {
		username: Set(username),
		name: Set(name),
		password: Set(password),
		..Default::default()
	}
	.insert(db)
	.await
}

/// Tombstone a live user so their messages continue to resolve to an author.
pub async fn tombstone_user_by_username(
	db: &DatabaseConnection,
	username: &str,
) -> Result<Option<User>, DbErr> {
	let mut updated = users::Entity::update_many()
		.set(users::ActiveModel {
			password: Set(String::new()),
			deleted_at: Set(Some(Utc::now().into())),
			..Default::default()
		})
		.filter(users::Column::Username.eq(username))
		.filter(users::Column::DeletedAt.is_null())
		.exec_with_returning(db)
		.await?;

	Ok(updated.pop())
}

pub async fn find_directory_by_id(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Option<Directory>, DbErr> {
	directory::Entity::find_by_id(id).one(db).await
}

pub async fn load_directory_tree(
	db: &DatabaseConnection,
	root: Directory,
) -> Result<Vec<Directory>, DbErr> {
	let mut queue = VecDeque::from([root.id]);
	let mut results = vec![root];

	while let Some(current_parent_id) = queue.pop_front() {
		let children = directory::Entity::find()
			.filter(directory::Column::ParentId.eq(Some(current_parent_id)))
			.all(db)
			.await?;

		for child in children {
			queue.push_back(child.id);
			results.push(child);
		}
	}

	Ok(results)
}

pub async fn insert_directory(
	db: &DatabaseConnection,
	name: String,
	kind: String,
	parent_id: Option<i32>,
) -> Result<Directory, DbErr> {
	directory::ActiveModel {
		name: Set(name),
		r#type: Set(kind),
		parent_id: Set(parent_id),
		..Default::default()
	}
	.insert(db)
	.await
}

/// Hard delete; cascading foreign keys remove descendants and messages.
pub async fn delete_directory_by_id(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Option<Directory>, DbErr> {
	let mut deleted = directory::Entity::delete_by_id(id)
		.exec_with_returning(db)
		.await?;
	Ok(deleted.pop())
}

pub async fn list_messages_by_directory(
	db: &DatabaseConnection,
	directory_id: i32,
) -> Result<Vec<Message>, DbErr> {
	messages::Entity::find()
		.filter(messages::Column::DirectoryId.eq(directory_id))
		.all(db)
		.await
}

pub async fn find_message_by_id(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Option<Message>, DbErr> {
	messages::Entity::find_by_id(id).one(db).await
}

pub async fn insert_message(
	db: &DatabaseConnection,
	author_username: String,
	content: String,
	directory_id: i32,
	parent_id: Option<i32>,
) -> Result<Message, DbErr> {
	messages::ActiveModel {
		author_username: Set(author_username),
		content: Set(content),
		directory_id: Set(directory_id),
		parent_id: Set(parent_id),
		created_at: Set(Utc::now().into()),
		..Default::default()
	}
	.insert(db)
	.await
}

/// Tombstone a live message owned by the authenticated user.
pub async fn tombstone_message_by_id(
	db: &DatabaseConnection,
	id: i32,
	author_username: &str,
) -> Result<Option<Message>, DbErr> {
	let mut updated = messages::Entity::update_many()
		.set(messages::ActiveModel {
			content: Set(String::new()),
			deleted_at: Set(Some(Utc::now().into())),
			..Default::default()
		})
		.filter(messages::Column::Id.eq(id))
		.filter(messages::Column::AuthorUsername.eq(author_username))
		.filter(messages::Column::DeletedAt.is_null())
		.exec_with_returning(db)
		.await?;

	Ok(updated.pop())
}
