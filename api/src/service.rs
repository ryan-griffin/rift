use crate::auth::{
	AuthSession, Credentials, PasswordError, generate_token, has_valid_token_shape, hash_password,
	hash_token, session_expires_at, validate_password, verify_password,
};
use crate::db;
use crate::entity::{
	directory::Model as Directory, messages::Model as Message, users::Model as User,
};
use crate::error::ServiceError;
use anyhow::Error;
use sea_orm::{
	AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, IsolationLevel,
	TransactionTrait,
};
use serde::{Deserialize, Serialize};

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

/// What a successful signup or login hands back: the user row plus the
/// single copy of the raw session token in existence.
#[derive(Serialize)]
pub struct AuthResponse {
	pub user: User,
	pub token: String,
}

struct PreparedSignup {
	username: String,
	name: String,
	password_hash: String,
}

async fn prepare_signup(input: SignupInput) -> Result<PreparedSignup, ServiceError> {
	let input = input.validate()?;
	validate_password(&input.password).map_err(|err| match err {
		PasswordError::Empty => ServiceError::BadRequest("password must not be empty".into()),
		PasswordError::TooLong => {
			ServiceError::BadRequest("password must be at most 72 bytes".into())
		}
	})?;

	let SignupInput {
		username,
		name,
		password,
	} = input;
	let password_hash = tokio::task::spawn_blocking(move || hash_password(&password))
		.await
		.map_err(|err| {
			ServiceError::Internal(Error::from(err).context("Password hashing task failed"))
		})?
		.map_err(|err| {
			ServiceError::Internal(Error::from(err).context("Failed to hash password"))
		})?;

	Ok(PreparedSignup {
		username,
		name,
		password_hash,
	})
}

pub async fn authenticate_user(
	db: &impl ConnectionTrait,
	credentials: &Credentials,
) -> Result<Option<User>, ServiceError> {
	let Some(user) = db::find_user_by_username(db, &credentials.username)
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to find user for authentication"),
			)
		})?
	else {
		return Ok(None);
	};

	// Deleted accounts authenticate as failures, same as a wrong
	// password: the wiped hash can never match anyway, but this
	// makes the policy explicit rather than incidental.
	if user.deleted_at.is_some() {
		return Ok(None);
	}

	let password = credentials.password.clone();
	let password_hash = user.password.clone();
	let matches = tokio::task::spawn_blocking(move || verify_password(&password, &password_hash))
		.await
		.map_err(|err| {
			ServiceError::Internal(Error::from(err).context("Password verification task failed"))
		})?
		.map_err(|err| {
			ServiceError::Internal(Error::from(err).context("Failed to verify password"))
		})?;

	Ok(matches.then_some(user))
}

/// Resolve a bearer token to its session. Malformed tokens short-circuit
/// before the database; unknown, expired, or tombstoned sessions resolve
/// to `None` so callers answer `Unauthorized` without distinguishing why.
pub async fn authenticate_token(
	db: &impl ConnectionTrait,
	token: &str,
) -> Result<Option<AuthSession>, ServiceError> {
	// Trim here rather than at each call site so header and query tokens
	// get the same treatment before shape validation.
	let token = token.trim();
	if !has_valid_token_shape(token) {
		return Ok(None);
	}

	Ok(db::find_active_auth_session(db, &hash_token(token))
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to look up authentication session"),
			)
		})?
		.map(|session| AuthSession {
			username: session.username,
			token_hash: session.token_hash,
			expires_at: session.expires_at,
		}))
}

pub async fn is_session_active(
	db: &impl ConnectionTrait,
	session: &AuthSession,
) -> Result<bool, ServiceError> {
	Ok(db::find_active_auth_session(db, &session.token_hash)
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to revalidate authentication session"),
			)
		})?
		.is_some_and(|active| active.username == session.username))
}

/// Mint a session row for `username` and return the raw token. Only the
/// hash is stored, so this is the single point where the secret exists.
pub async fn issue_token(
	db: &impl ConnectionTrait,
	username: &str,
) -> Result<String, ServiceError> {
	let token = generate_token().map_err(ServiceError::Internal)?;
	let expires_at = session_expires_at().map_err(ServiceError::Internal)?;

	db::insert_auth_session(db, hash_token(&token), username.to_owned(), expires_at)
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to store authentication session"),
			)
		})?;

	Ok(token)
}

pub async fn cleanup_expired_auth_sessions(db: &DatabaseConnection) -> Result<u64, ServiceError> {
	db::delete_expired_auth_sessions(db).await.map_err(|err| {
		ServiceError::Internal(
			Error::from(err).context("Failed to delete expired authentication sessions"),
		)
	})
}

pub async fn revoke_session(
	db: &impl ConnectionTrait,
	token_hash: &str,
) -> Result<(), ServiceError> {
	db::delete_auth_session(db, token_hash)
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to revoke authentication session"),
			)
		})?;
	Ok(())
}

pub async fn revoke_user_sessions(
	db: &impl ConnectionTrait,
	username: &str,
) -> Result<(), ServiceError> {
	db::delete_auth_sessions_by_username(db, username)
		.await
		.map_err(|err| {
			ServiceError::Internal(
				Error::from(err).context("Failed to revoke user authentication sessions"),
			)
		})?;
	Ok(())
}

/// Commit `result`'s transaction, or roll back and return the original
/// error. Dropping the transaction would roll back silently; the explicit
/// rollback only adds logging when rollback itself fails.
async fn finish_transaction<T>(
	txn: DatabaseTransaction,
	result: Result<T, ServiceError>,
	op: &str,
) -> Result<T, ServiceError> {
	match result {
		Ok(value) => {
			txn.commit().await?;
			Ok(value)
		}
		Err(err) => {
			if let Err(rollback_err) = txn.rollback().await {
				eprintln!("Failed to roll back {op} transaction: {rollback_err:?}");
			}
			Err(err)
		}
	}
}

pub async fn signup(
	db: &DatabaseConnection,
	input: SignupInput,
) -> Result<AuthResponse, ServiceError> {
	let input = prepare_signup(input).await?;
	let txn = db.begin().await?;
	let result = async {
		let created_user =
			db::insert_user(&txn, input.username, input.name, input.password_hash).await?;
		let token = issue_token(&txn, &created_user.username).await?;
		Ok(AuthResponse {
			user: created_user,
			token,
		})
	}
	.await;

	finish_transaction(txn, result, "signup").await
}

pub async fn login(
	db: &DatabaseConnection,
	mut credentials: Credentials,
) -> Result<AuthResponse, ServiceError> {
	// Keep credential failures opaque so login does not reveal which field
	// was invalid or whether the account exists.
	credentials.username =
		normalize_username(credentials.username).map_err(|_| ServiceError::Unauthorized)?;
	validate_password(&credentials.password).map_err(|_| ServiceError::Unauthorized)?;

	let user = authenticate_user(db, &credentials)
		.await?
		.ok_or(ServiceError::Unauthorized)?;
	let token = issue_token(db, &user.username).await?;

	Ok(AuthResponse { user, token })
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

	let txn = db.begin().await?;
	let result = async {
		if let Some(user) = db::tombstone_user_by_username(&txn, authenticated_username).await? {
			revoke_user_sessions(&txn, authenticated_username).await?;
			return Ok(user);
		}

		match db::find_user_by_username(&txn, authenticated_username).await? {
			Some(user) if user.deleted_at.is_some() => Err(ServiceError::Gone(format!(
				"User with username {authenticated_username} is already deleted"
			))),
			Some(_) => Err(ServiceError::Conflict(format!(
				"User with username {authenticated_username} changed concurrently; retry the request"
			))),
			None => Err(ServiceError::NotFound(format!(
				"User with username {authenticated_username} not found"
			))),
		}
	}
	.await;

	finish_transaction(txn, result, "delete user").await
}

pub async fn get_directory(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Vec<Directory>, ServiceError> {
	let id = validate_id(id, "directory")?;
	let directories = db::load_directory_tree(db, id).await?;
	if directories.is_empty() {
		return Err(ServiceError::NotFound(format!(
			"Directory with id {id} not found"
		)));
	}
	Ok(directories)
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
	let id = validate_id(id, "directory")?;
	db::delete_directory_by_id(db, id)
		.await?
		.ok_or_else(|| ServiceError::NotFound(format!("Directory with id {id} not found")))
}

pub async fn get_message_thread(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Vec<Message>, ServiceError> {
	let txn = db
		.begin_with_config(
			Some(IsolationLevel::RepeatableRead),
			Some(AccessMode::ReadOnly),
		)
		.await?;
	let result = async {
		let thread = require_thread(&txn, id).await?;
		Ok(db::list_messages_by_directory(&txn, thread.id).await?)
	}
	.await;

	finish_transaction(txn, result, "get message thread").await
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

	let Some(parent_id) = input.parent_id else {
		require_thread(db, input.directory_id).await?;
		return Ok(db::insert_message(
			db,
			author_username,
			input.content,
			input.directory_id,
			None,
		)
		.await?);
	};

	let txn = db.begin().await?;
	let result = async {
		let directory = db::find_directory_by_id_for_key_share(&txn, input.directory_id)
			.await?
			.ok_or_else(|| {
				ServiceError::NotFound(format!(
					"Directory with id {} not found",
					input.directory_id
				))
			})?;

		if directory.r#type != "thread" {
			return Err(ServiceError::BadRequest(format!(
				"Directory {} is a '{}', not a thread",
				directory.id, directory.r#type
			)));
		}

		let parent = db::find_message_by_id_for_share(&txn, parent_id)
			.await?
			.ok_or_else(|| {
				ServiceError::NotFound(format!("Parent message with id {parent_id} not found"))
			})?;

		if parent.deleted_at.is_some() {
			return Err(ServiceError::Conflict(format!(
				"Parent message {parent_id} is deleted"
			)));
		}
		if parent.directory_id != input.directory_id {
			return Err(ServiceError::BadRequest(format!(
				"Parent message {parent_id} belongs to another thread"
			)));
		}

		Ok(db::insert_message(
			&txn,
			author_username,
			input.content,
			input.directory_id,
			Some(parent_id),
		)
		.await?)
	}
	.await;

	finish_transaction(txn, result, "create message").await
}

pub async fn delete_message(
	db: &DatabaseConnection,
	authenticated_username: &str,
	id: i32,
) -> Result<Message, ServiceError> {
	let id = validate_id(id, "message")?;
	if let Some(message) = db::tombstone_message_by_id(db, id, authenticated_username).await? {
		return Ok(message);
	}

	match db::find_message_by_id(db, id).await? {
		None => Err(ServiceError::NotFound(format!(
			"Message with id {id} not found"
		))),
		Some(message) if message.author_username != authenticated_username => Err(
			ServiceError::Forbidden("You can only delete your own messages".into()),
		),
		Some(message) if message.deleted_at.is_some() => Err(ServiceError::Gone(format!(
			"Message with id {id} is already deleted"
		))),
		Some(_) => Err(ServiceError::Conflict(format!(
			"Message with id {id} changed concurrently; retry the request"
		))),
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

async fn require_directory(db: &impl ConnectionTrait, id: i32) -> Result<Directory, ServiceError> {
	db::find_directory_by_id(db, id)
		.await?
		.ok_or_else(|| ServiceError::NotFound(format!("Directory with id {id} not found")))
}

pub async fn require_thread(db: &impl ConnectionTrait, id: i32) -> Result<Directory, ServiceError> {
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
