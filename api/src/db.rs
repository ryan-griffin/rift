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
	// Signup trims username, so we must do so here.
	let username = username.trim();
	users::Entity::find()
		.filter(users::Column::Username.eq(username))
		.one(db)
		.await?
		.ok_or(DbErr::RecordNotFound(format!(
			"User with username {username} not found"
		)))
}

pub async fn create_user(db: &DatabaseConnection, user: User) -> Result<User, DbErr> {
	// Password should not be trimmed.
	users::ActiveModel {
		username: Set(user.username.trim().to_string()),
		name: Set(user.name.trim().to_string()),
		password: Set(user.password),
		..Default::default()
	}
	.insert(db)
	.await
}

/// Tombstone a user: the row stays so their messages keep resolving to an
/// author, the password is wiped so login can never succeed.
/// Reversible as identity, not as auth — un-deleting requires a password reset.
pub async fn delete_user(db: &DatabaseConnection, username: &str) -> Result<User, DbErr> {
	let user = users::Entity::find_by_id(username.trim())
		.one(db)
		.await?
		.ok_or(DbErr::RecordNotFound(format!(
			"User with username {username} not found"
		)))?;

	if user.deleted_at.is_some() {
		return Ok(user);
	}

	users::ActiveModel {
		username: Set(user.username),
		password: Set(String::new()),
		deleted_at: Set(Some(Utc::now().into())),
		..Default::default()
	}
	.update(db)
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

pub async fn create_directory(
	db: &DatabaseConnection,
	directory: Directory,
) -> Result<Directory, DbErr> {
	// FK ensures a parent exists but can't inspect its type. Folders must be
	// matched explicitly so types added later default to leaf.
	if let Some(parent_id) = directory.parent_id {
		let parent = directory::Entity::find_by_id(parent_id)
			.one(db)
			.await?
			.ok_or(DbErr::RecordNotFound(format!(
				"Directory with id {parent_id} not found"
			)))?;

		if parent.r#type != "folder" {
			return Err(DbErr::Custom(format!(
				"Directory {parent_id} is not a folder and cannot have children"
			)));
		}
	}

	directory::ActiveModel {
		name: Set(directory.name.trim().to_string()),
		r#type: Set(directory.r#type),
		parent_id: Set(directory.parent_id),
		..Default::default()
	}
	.insert(db)
	.await
}

pub async fn get_message_thread(db: &DatabaseConnection, id: i32) -> Result<Vec<Message>, DbErr> {
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
	// The target must exist and be a thread; same-thread replies are
	// enforced by fk_messages_parent_same_thread in m3. Replying to a
	// tombstoned parent is rejected here — the FK can't see deleted_at.
	match directory::Entity::find_by_id(message.directory_id)
		.one(db)
		.await?
	{
		Some(directory) if directory.r#type == "thread" => {
			if let Some(parent_id) = message.parent_id {
				let parent = messages::Entity::find_by_id(parent_id)
					.one(db)
					.await?
					.ok_or(DbErr::RecordNotFound(format!(
						"Parent message with id {parent_id} not found"
					)))?;

				if parent.deleted_at.is_some() {
					return Err(DbErr::Custom(format!(
						"Parent message {parent_id} is deleted"
					)));
				}
			}

			messages::ActiveModel {
				author_username: Set(author_username),
				// Content is stored verbatim; whitespace can be deliberate.
				content: Set(message.content),
				directory_id: Set(message.directory_id),
				parent_id: Set(message.parent_id),
				created_at: Set(Utc::now().into()),
				..Default::default()
			}
			.insert(db)
			.await
		}
		Some(directory) => Err(DbErr::Custom(format!(
			"Messages can only be created for a directory node of type 'thread', not '{}'",
			directory.r#type
		))),
		None => Err(DbErr::RecordNotFound(format!(
			"Directory with id {} not found",
			message.directory_id
		))),
	}
}

/// Tombstone a message: content is wiped, the row and reply links stay.
/// Idempotent. Ownership gating happens in callers.
pub async fn delete_message(db: &DatabaseConnection, id: i32) -> Result<Message, DbErr> {
	let message = messages::Entity::find_by_id(id)
		.one(db)
		.await?
		.ok_or(DbErr::RecordNotFound(format!(
			"Message with id {id} not found"
		)))?;

	if message.deleted_at.is_some() {
		return Ok(message);
	}

	messages::ActiveModel {
		id: Set(message.id),
		content: Set(String::new()),
		deleted_at: Set(Some(Utc::now().into())),
		..Default::default()
	}
	.update(db)
	.await
}
