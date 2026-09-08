use crate::entity::{
	auth_sessions, auth_sessions::Model as AuthSession, directory, directory::Model as Directory,
	messages, messages::Model as Message, users, users::Model as User,
};
use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr,
	EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
	sea_query::{
		Alias, CommonTableExpression, Expr, JoinType, LockType, Query, SelectStatement, UnionType,
		WithClause,
	},
};

pub async fn list_users(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
	users::Entity::find().all(db).await
}

pub async fn find_user_by_username(
	db: &impl ConnectionTrait,
	username: &str,
) -> Result<Option<User>, DbErr> {
	users::Entity::find()
		.filter(users::Column::Username.eq(username))
		.one(db)
		.await
}

pub async fn insert_user(
	db: &impl ConnectionTrait,
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
	db: &impl ConnectionTrait,
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

pub async fn insert_auth_session(
	db: &impl ConnectionTrait,
	token_hash: String,
	username: String,
	expires_at: chrono::DateTime<chrono::FixedOffset>,
) -> Result<AuthSession, DbErr> {
	auth_sessions::ActiveModel {
		token_hash: Set(token_hash),
		username: Set(username),
		expires_at: Set(expires_at),
	}
	.insert(db)
	.await
}

/// A session counts as active only while it is unexpired *and* its user
/// has not been tombstoned. The join enforces the second half: deleting
/// an account implicitly revokes every session, on top of the explicit
/// row deletes issued at logout and account deletion.
pub async fn find_active_auth_session(
	db: &impl ConnectionTrait,
	token_hash: &str,
) -> Result<Option<AuthSession>, DbErr> {
	let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
	auth_sessions::Entity::find_by_id(token_hash)
		.join(JoinType::InnerJoin, auth_sessions::Relation::Users.def())
		.filter(auth_sessions::Column::ExpiresAt.gt(now))
		.filter(users::Column::DeletedAt.is_null())
		.one(db)
		.await
}

pub async fn delete_auth_session(
	db: &impl ConnectionTrait,
	token_hash: &str,
) -> Result<u64, DbErr> {
	Ok(auth_sessions::Entity::delete_by_id(token_hash)
		.exec(db)
		.await?
		.rows_affected)
}

pub async fn delete_auth_sessions_by_username(
	db: &impl ConnectionTrait,
	username: &str,
) -> Result<u64, DbErr> {
	Ok(auth_sessions::Entity::delete_many()
		.filter(auth_sessions::Column::Username.eq(username))
		.exec(db)
		.await?
		.rows_affected)
}

pub async fn delete_expired_auth_sessions(db: &impl ConnectionTrait) -> Result<u64, DbErr> {
	let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
	Ok(auth_sessions::Entity::delete_many()
		.filter(auth_sessions::Column::ExpiresAt.lte(now))
		.exec(db)
		.await?
		.rows_affected)
}

pub async fn find_directory_by_id(
	db: &impl ConnectionTrait,
	id: i32,
) -> Result<Option<Directory>, DbErr> {
	directory::Entity::find_by_id(id).one(db).await
}

pub async fn find_directory_by_id_for_key_share(
	db: &DatabaseTransaction,
	id: i32,
) -> Result<Option<Directory>, DbErr> {
	directory::Entity::find_by_id(id)
		.lock(LockType::KeyShare)
		.one(db)
		.await
}

fn directory_tree_query(root_id: i32) -> SelectStatement {
	let tree = Alias::new("directory_tree");
	let columns = [
		directory::Column::Id,
		directory::Column::Name,
		directory::Column::Type,
		directory::Column::ParentId,
	];

	let base = Query::select()
		.columns(columns.map(|column| (directory::Entity, column)))
		.from(directory::Entity)
		.and_where(Expr::col((directory::Entity, directory::Column::Id)).eq(root_id))
		.to_owned();

	let recursive = Query::select()
		.columns(columns.map(|column| (directory::Entity, column)))
		.from(directory::Entity)
		.join(
			JoinType::InnerJoin,
			tree.clone(),
			Expr::col((directory::Entity, directory::Column::ParentId))
				.equals((tree.clone(), directory::Column::Id)),
		)
		.to_owned();

	let cte = CommonTableExpression::new()
		.query(
			base.clone()
				.union(UnionType::Distinct, recursive)
				.to_owned(),
		)
		.columns(columns)
		.table_name(tree.clone())
		.to_owned();
	let with_clause = WithClause::new().recursive(true).cte(cte).to_owned();
	let mut query = Query::select()
		.columns(columns.map(|column| (tree.clone(), column)))
		.from(tree)
		.to_owned();
	query.with_cte(with_clause);
	query
}

pub async fn load_directory_tree(
	db: &DatabaseConnection,
	root_id: i32,
) -> Result<Vec<Directory>, DbErr> {
	let statement = db
		.get_database_backend()
		.build(&directory_tree_query(root_id));
	directory::Entity::find()
		.from_raw_sql(statement)
		.all(db)
		.await
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
	db: &impl ConnectionTrait,
	directory_id: i32,
) -> Result<Vec<Message>, DbErr> {
	messages::Entity::find()
		.filter(messages::Column::DirectoryId.eq(directory_id))
		.order_by_asc(messages::Column::CreatedAt)
		.order_by_asc(messages::Column::Id)
		.all(db)
		.await
}

pub async fn find_message_by_id(
	db: &DatabaseConnection,
	id: i32,
) -> Result<Option<Message>, DbErr> {
	messages::Entity::find_by_id(id).one(db).await
}

pub async fn find_message_by_id_for_share(
	db: &DatabaseTransaction,
	id: i32,
) -> Result<Option<Message>, DbErr> {
	messages::Entity::find_by_id(id).lock_shared().one(db).await
}

pub async fn insert_message(
	db: &impl ConnectionTrait,
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
