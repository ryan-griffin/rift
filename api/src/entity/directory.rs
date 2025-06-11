use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "directory")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	pub name: String,
	pub r#type: String,
	pub parent_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "Entity",
		from = "Column::ParentId",
		to = "Column::Id",
		on_update = "Cascade",
		on_delete = "SetNull"
	)]
	SelfRef,
}

impl ActiveModelBehavior for ActiveModel {}
