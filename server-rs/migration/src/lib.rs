pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_users_table;

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Email,
    DisplayName,
    ImageUrl,
    CreatedAt,
    UpdatedAt,
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250101_000001_create_users_table::Migration)]
    }
}
