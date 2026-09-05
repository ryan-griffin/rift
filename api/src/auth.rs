use anyhow::{Context, Result};
use bcrypt::BcryptError;
use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt::Write;

const TOKEN_BYTES: usize = 32;
const TOKEN_LENGTH: usize = TOKEN_BYTES * 2;
const SESSION_LIFETIME_DAYS: i64 = 30;

#[derive(Clone, Debug)]
pub struct AuthSession {
	pub username: String,
	pub token_hash: String,
	pub expires_at: DateTime<FixedOffset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
	pub username: String,
	pub password: String,
}

/// Why a password failed validation; transport mapping happens in callers.
#[derive(Debug)]
pub enum PasswordError {
	Empty,
	TooLong,
}

/// bcrypt ignores input past 72 bytes and empty passwords are never valid,
/// so both are rejected here rather than silently accepted.
pub fn validate_password(password: &str) -> Result<(), PasswordError> {
	if password.is_empty() {
		return Err(PasswordError::Empty);
	}
	if password.len() > 72 {
		return Err(PasswordError::TooLong);
	}
	Ok(())
}

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
	bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, BcryptError> {
	bcrypt::verify(password, hash)
}

fn encode_hex(bytes: &[u8]) -> String {
	let mut encoded = String::with_capacity(bytes.len() * 2);
	for byte in bytes {
		write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
	}
	encoded
}

pub fn hash_token(token: &str) -> String {
	encode_hex(&Sha256::digest(token.as_bytes()))
}

pub fn has_valid_token_shape(token: &str) -> bool {
	token.len() == TOKEN_LENGTH
		&& token
			.bytes()
			.all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A fresh opaque session token: 256 bits of randomness, hex encoded.
/// Storage and expiry live in the service layer; this is just the secret.
pub fn generate_token() -> Result<String> {
	let mut token_bytes = [0_u8; TOKEN_BYTES];
	getrandom::fill(&mut token_bytes)
		.map_err(|err| anyhow::anyhow!("Failed to generate an authentication token: {err}"))?;
	Ok(encode_hex(&token_bytes))
}

pub fn session_expires_at() -> Result<DateTime<FixedOffset>> {
	Ok(Utc::now()
		.checked_add_signed(chrono::Duration::days(SESSION_LIFETIME_DAYS))
		.context("Failed to calculate session expiration time")?
		.into())
}
