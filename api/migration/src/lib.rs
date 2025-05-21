pub use sea_orm_migration::prelude::*;

mod m20250520_213905_create_users_table;
mod m20250520_213906_create_directory_table;
mod m20250520_222700_seed;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250520_213905_create_users_table::Migration),
            Box::new(m20250520_213906_create_directory_table::Migration),
            Box::new(m20250520_222700_seed::Migration),
        ]
    }
}
