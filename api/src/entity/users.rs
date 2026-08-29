use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub username: String,
	pub name: String,
	#[serde(skip_serializing)]
	pub password: String,
	pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
