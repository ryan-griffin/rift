use crate::db;
use crate::entity;
use axum::{
	extract::Request,
	http::{HeaderMap, StatusCode},
	middleware::Next,
	response::Response,
};
use entity::users::Model as User;
use jsonwebtoken::{
	DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode, errors::Error,
};
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
struct Claims {
	pub sub: String, // username
	pub exp: usize,  // expiration time
}

#[derive(Deserialize)]
pub struct Credentials {
	pub username: String,
	// TODO: Add password field
	// pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
	pub user: User,
	pub token: String,
}

pub fn generate_token(username: &str) -> Result<String, Error> {
	let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

	let expiration = chrono::Utc::now()
		.checked_add_signed(chrono::Duration::hours(24))
		.expect("valid timestamp")
		.timestamp() as usize;

	let claims = Claims {
		sub: username.to_string(),
		exp: expiration,
	};

	encode(
		&Header::default(),
		&claims,
		&EncodingKey::from_secret(jwt_secret.as_ref()),
	)
}

fn validate_token(token: &str) -> Result<Claims, Error> {
	let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

	let token_data: TokenData<Claims> = decode(
		token,
		&DecodingKey::from_secret(jwt_secret.as_ref()),
		&Validation::default(),
	)?;

	Ok(token_data.claims)
}

fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
	headers
		.get("Authorization")?
		.to_str()
		.ok()?
		.strip_prefix("Bearer ")
		.map(|token| token.to_string())
}

pub async fn authenticate_user(
	db: &DatabaseConnection,
	credentials: &Credentials,
) -> Result<Option<User>, DbErr> {
	// TODO: Verify password here
	match db::get_user(db, &credentials.username).await {
		Ok(user) => Ok(Some(user)),
		Err(DbErr::RecordNotFound(_)) => Ok(None),
		Err(err) => Err(err),
	}
}

pub async fn auth_middleware(
	headers: HeaderMap,
	mut request: Request,
	next: Next,
) -> Result<Response, StatusCode> {
	let token = extract_token_from_header(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
	let claims = validate_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;

	// Add the username to request extensions so handlers can access it
	request.extensions_mut().insert(claims.sub);

	Ok(next.run(request).await)
}
