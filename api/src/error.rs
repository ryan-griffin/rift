use anyhow::Error as AnyhowError;
use sea_orm::{DbErr, RuntimeErr, SqlxError};

/// Transport-agnostic API error: the client-vs-internal classification
/// lives here once. `routes.rs` renders it for REST via `IntoResponse`;
/// the websocket layer converts it into its own client/internal error.
#[derive(Debug)]
pub enum ServiceError {
	NotFound(String),
	BadRequest(String),
	Conflict(String),
	Forbidden(String),
	Unauthorized,
	Internal(AnyhowError),
}

impl From<DbErr> for ServiceError {
	fn from(err: DbErr) -> Self {
		match &err {
			DbErr::Exec(RuntimeErr::SqlxError(SqlxError::Database(e)))
			| DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(e))) => {
				match e.code().as_deref() {
					// Postgres SQLSTATEs: 23505 unique_violation, 23503
					// foreign_key_violation, 23514 check_violation, 22001
					// string_data_right_truncation.
					Some("23505") => {
						Self::Conflict("An entry with these details already exists".into())
					}
					Some("23503") => {
						Self::BadRequest("Related record is missing or invalid".into())
					}
					Some("23514") => Self::BadRequest("Value violates a data constraint".into()),
					Some("22001") => Self::BadRequest("Value is too long".into()),
					_ => Self::Internal(err.into()),
				}
			}
			_ => Self::Internal(err.into()),
		}
	}
}

impl From<AnyhowError> for ServiceError {
	fn from(err: AnyhowError) -> Self {
		Self::Internal(err)
	}
}
