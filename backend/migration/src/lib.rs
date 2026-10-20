//! 院级科研申报数据库迁移。

pub use sea_orm_migration::prelude::*;

mod m20260929_000001_init_schema;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260929_000001_init_schema::Migration)]
    }
}
