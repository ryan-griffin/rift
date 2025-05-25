pub use sea_orm_migration::prelude::*;

mod m1_create_users_table;
mod m2_create_directory_table;
mod m3_create_messages_table;
mod m99_seed;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m1_create_users_table::Migration),
            Box::new(m2_create_directory_table::Migration),
            Box::new(m3_create_messages_table::Migration),
            Box::new(m99_seed::Migration),
        ]
    }
}
