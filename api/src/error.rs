use anyhow::Error as AnyhowError;
use sea_orm::{DbErr, RuntimeErr, SqlxError};

/// Transport-agnostic API error: the client-vs-internal classification
/// lives here once. `routes.rs` renders it for REST via `IntoResponse`;
/// the websocket layer converts it into its own client/internal error.
#[derive(Debug)]
pub enum ApiError {
	NotFound(String),
	BadRequest(String),
	Conflict(String),
	Forbidden(String),
	Unauthorized,
	Internal(AnyhowError),
}

impl ApiError {
	/// 403 — a non-author tried to delete a message.
	pub fn only_own_messages() -> Self {
		Self::Forbidden("You can only delete your own messages".into())
	}

	/// 403 — a user tried to delete someone else's account.
	pub fn only_own_account() -> Self {
		Self::Forbidden("You can only delete your own account".into())
	}
}

impl From<DbErr> for ApiError {
	fn from(err: DbErr) -> Self {
		match &err {
			// Mostly messages from db.rs, but note sea-orm itself also
			// constructs RecordNotFound with fixed internal strings (e.g.
			// "Failed to find updated item" when a row vanishes between
			// find and update) — harmless to echo, worth knowing on
			// upgrades.
			DbErr::RecordNotFound(msg) => Self::NotFound(msg.clone()),
			// Safe to echo only because db.rs's validation sites are the
			// sole producers of DbErr::Custom; sea-orm itself never
			// constructs it (checked against sea-orm 1.1.19). Re-verify on
			// upgrades before extending this arm.
			DbErr::Custom(msg) => Self::BadRequest(msg.clone()),
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

impl From<AnyhowError> for ApiError {
	fn from(err: AnyhowError) -> Self {
		Self::Internal(err)
	}
}
