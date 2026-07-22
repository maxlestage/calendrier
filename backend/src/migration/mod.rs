use sea_orm_migration::prelude::*;

mod m20240101_000001_create_event_table;
mod m20260722_000001_create_setting_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_event_table::Migration),
            Box::new(m20260722_000001_create_setting_table::Migration),
        ]
    }
}
