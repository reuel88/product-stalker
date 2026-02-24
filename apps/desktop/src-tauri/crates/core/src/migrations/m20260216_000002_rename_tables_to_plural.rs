use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Rename exchange_rate → exchange_rates (only if old singular name exists)
        let has_singular = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='exchange_rate'"
                    .to_owned(),
            ))
            .await?
            .is_some();

        if has_singular {
            db.execute_unprepared("ALTER TABLE exchange_rate RENAME TO exchange_rates")
                .await?;
        }

        // Rename verified_session → verified_sessions (only if old singular name exists)
        let has_singular = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='verified_session'"
                    .to_owned(),
            ))
            .await?
            .is_some();

        if has_singular {
            db.execute_unprepared("ALTER TABLE verified_session RENAME TO verified_sessions")
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op: we don't want to revert to singular names
        Ok(())
    }
}
