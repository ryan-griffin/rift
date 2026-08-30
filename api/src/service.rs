use crate::auth::{
	Credentials, PasswordError, authenticate_user, hash_password, validate_password,
};
use crate::db;
use crate::entity::{
	directory::Model as Directory, messages::Model as Message, users::Model as User,
};
use crate::error::ServiceError;
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignupInput {
	username: String,
	name: String,
	password: String,
}

impl SignupInput {
	fn validate(mut self) -> Result<Self, ServiceError> {
		self.username = normalize_username(self.username)?;
		self.name = self.name.trim().to_owned();
		validate_name(&self.name, "name")?;
		Ok(self)
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DirectoryType {
	Folder,
	Thread,
}

impl DirectoryType {
	fn as_str(&self) -> &'static str {
		match self {
			Self::Folder => "folder",
			Self::Thread => "thread",
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDirectoryInput {
	name: String,
	#[serde(rename = "type")]
	kind: DirectoryType,
	parent_id: Option<i32>,
}

impl CreateDirectoryInput {
	fn validate(mut self) -> Result<Self, ServiceError> {
		self.name = self.name.trim().to_owned();
		validate_name(&self.name, "directory name")?;
		if let Some(parent_id) = self.parent_id {
			validate_id(parent_id, "parent directory")?;
		}
		Ok(self)
	}
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateMessageInput {
	content: String,
	directory_id: i32,
	parent_id: Option<i32>,
}

impl CreateMessageInput {
	fn validate(self) -> Result<Self, ServiceError> {
		if self.content.trim().is_empty() {
			return Err(ServiceError::BadRequest(
				"message content must not be empty".into(),
			));
		}
		if self.content.chars().count() > 4000 {
			return Err(ServiceError::BadRequest(
				"message content must be at most 4000 characters".into(),
			));
		}
		if self.content.contains('\0') {
			return Err(ServiceError::BadRequest(
				"message content must not contain null characters".into(),
			));
		}

		validate_id(self.directory_id, "directory")?;
		if let Some(parent_id) = self.parent_id {
			validate_id(parent_id, "parent message")?;
		}
		Ok(self)
	}
}

pub async fn get_users(db: &DatabaseConnection) -> Result<Vec<User>, ServiceError> {
	Ok(db::list_users(db).await?)
}

pub async fn get_user(db: &DatabaseConnection, username: String) -> Result<User, ServiceError> {
	let username = normalize_username(username)?;
	require_user(db, &username).await
}

pub async fn create_user(
	db: &DatabaseConnection,
	input: SignupInput,
) -> Result<User, ServiceError> {
	let mut input = input.validate()?;
	validate_password(&input.password).map_err(|err| match err {
		PasswordError::Empty => ServiceError::BadRequest("password must not be empty".into()),
		PasswordError::TooLong => {
			ServiceError::BadRequest("password must be at most 72 bytes".into())
		}
	})?;
	input.password =
		hash_password(&input.password).map_err(|err| ServiceError::Internal(err.into()))?;

	Ok(db::insert_user(db, input.username, input.name, input.password).await?)
}

pub async fn login(
	db: &DatabaseConnection,
	mut credentials: Credentials,
) -> Result<User, ServiceError> {
	// Keep credential failures opaque so login does not reveal which field
	// was invalid or whether the account exists.
	credentials.username =
		normalize_username(credentials.username).map_err(|_| ServiceError::Unauthorized)?;
	validate_password(&credentials.password).map_err(|_| ServiceError::Unauthorized)?;

	authenticate_user(db, &credentials)
		.await?
		.ok_or(ServiceError::Unauthorized)
}

pub async fn delete_user(
	db: &DatabaseConnection,
	authenticated_username: &str,
	requested_username: String,
) -> Result<User, ServiceError> {
	let requested_username = normalize_username(requested_username)?;
	if requested_username != authenticated_username {
		return Err(ServiceError::Forbidden(
			"You can only delete your own account".into(),
		));
	}

	let user = require_user(db, authenticated_username).await?;
	Ok(db::tombstone_user(db, user).await?)
}

pub async fn get_directory(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Vec<Directory>, ServiceError> {
	let directory = require_directory(db, validate_id(id, "directory")?).await?;
	Ok(db::load_directory_tree(db, directory).await?)
}

pub async fn create_directory(
	db: &DatabaseConnection,
	input: CreateDirectoryInput,
) -> Result<Directory, ServiceError> {
	let input = input.validate()?;

	if let Some(parent_id) = input.parent_id {
		let parent = require_directory(db, parent_id).await?;
		if parent.r#type != "folder" {
			return Err(ServiceError::BadRequest(format!(
				"Directory {parent_id} is not a folder and cannot have children"
			)));
		}
	}

	Ok(db::insert_directory(
		db,
		input.name,
		input.kind.as_str().to_owned(),
		input.parent_id,
	)
	.await?)
}

pub async fn delete_directory(db: &DatabaseConnection, id: i32) -> Result<Directory, ServiceError> {
	let directory = require_directory(db, validate_id(id, "directory")?).await?;
	db::delete_directory_by_id(db, directory.id).await?;
	Ok(directory)
}

pub async fn get_message_thread(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Vec<Message>, ServiceError> {
	let thread = require_thread(db, id).await?;
	Ok(db::list_messages_by_directory(db, thread.id).await?)
}

pub async fn get_message(db: &DatabaseConnection, id: i32) -> Result<Message, ServiceError> {
	let id = validate_id(id, "message")?;
	require_message(db, id).await
}

pub async fn create_message(
	db: &DatabaseConnection,
	author_username: String,
	input: CreateMessageInput,
) -> Result<Message, ServiceError> {
	let input = input.validate()?;
	require_thread(db, input.directory_id).await?;

	if let Some(parent_id) = input.parent_id {
		let parent = require_message(db, parent_id)
			.await
			.map_err(|err| match err {
				ServiceError::NotFound(_) => {
					ServiceError::NotFound(format!("Parent message with id {parent_id} not found"))
				}
				other => other,
			})?;

		if parent.deleted_at.is_some() {
			return Err(ServiceError::BadRequest(format!(
				"Parent message {parent_id} is deleted"
			)));
		}
		if parent.directory_id != input.directory_id {
			return Err(ServiceError::BadRequest(format!(
				"Parent message {parent_id} belongs to another thread"
			)));
		}
	}

	Ok(db::insert_message(
		db,
		author_username,
		input.content,
		input.directory_id,
		input.parent_id,
	)
	.await?)
}

pub async fn delete_message(
	db: &DatabaseConnection,
	authenticated_username: &str,
	id: i32,
) -> Result<Message, ServiceError> {
	let message = get_message(db, id).await?;
	if message.author_username != authenticated_username {
		return Err(ServiceError::Forbidden(
			"You can only delete your own messages".into(),
		));
	}

	match db::tombstone_message(db, message).await {
		Ok(message) => Ok(message),
		Err(DbErr::RecordNotUpdated) => Err(ServiceError::NotFound(format!(
			"Message with id {id} not found"
		))),
		Err(err) => Err(err.into()),
	}
}

pub fn validate_thread_id(id: i32) -> Result<i32, ServiceError> {
	validate_id(id, "thread")
}

fn normalize_username(username: String) -> Result<String, ServiceError> {
	let username = username.trim().to_owned();
	if username.is_empty() {
		return Err(ServiceError::BadRequest(
			"username must not be empty".into(),
		));
	}
	if !username
		.bytes()
		.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
	{
		return Err(ServiceError::BadRequest(
			"username may only contain lowercase letters, numbers, and underscores".into(),
		));
	}
	if username.len() > 32 {
		return Err(ServiceError::BadRequest(
			"username must be at most 32 characters".into(),
		));
	}
	Ok(username)
}

fn validate_id(id: i32, field: &str) -> Result<i32, ServiceError> {
	if id <= 0 {
		return Err(ServiceError::BadRequest(format!(
			"{field} id must be a positive integer"
		)));
	}
	Ok(id)
}

fn validate_name(value: &str, field: &str) -> Result<(), ServiceError> {
	if value.is_empty() {
		return Err(ServiceError::BadRequest(format!(
			"{field} must not be empty"
		)));
	}
	if value.chars().count() > 64 {
		return Err(ServiceError::BadRequest(format!(
			"{field} must be at most 64 characters"
		)));
	}
	if value.chars().any(char::is_control) {
		return Err(ServiceError::BadRequest(format!(
			"{field} must not contain control characters"
		)));
	}
	Ok(())
}

async fn require_user(db: &DatabaseConnection, username: &str) -> Result<User, ServiceError> {
	db::find_user_by_username(db, username)
		.await?
		.ok_or_else(|| ServiceError::NotFound(format!("User with username {username} not found")))
}

async fn require_directory(db: &DatabaseConnection, id: i32) -> Result<Directory, ServiceError> {
	db::find_directory_by_id(db, id)
		.await?
		.ok_or_else(|| ServiceError::NotFound(format!("Directory with id {id} not found")))
}

pub async fn require_thread(db: &DatabaseConnection, id: i32) -> Result<Directory, ServiceError> {
	let id = validate_thread_id(id)?;
	let directory = require_directory(db, id).await?;
	if directory.r#type != "thread" {
		return Err(ServiceError::BadRequest(format!(
			"Directory {id} is a '{}', not a thread",
			directory.r#type
		)));
	}
	Ok(directory)
}

async fn require_message(db: &DatabaseConnection, id: i32) -> Result<Message, ServiceError> {
	db::find_message_by_id(db, id)
		.await?
		.ok_or_else(|| ServiceError::NotFound(format!("Message with id {id} not found")))
}
