//! Adds columns missing from initial schema vs NodeAdmin standard:
//! - `roles.guard_name` (VARCHAR 20, default 'web')
//! - `settings.favicon` (VARCHAR 255, nullable)

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add guard_name to roles
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("roles"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("guard_name"))
                            .string_len(20)
                            .null()
                            .default("web"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add favicon to settings
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("settings"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("favicon")).string().null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite does not support DROP COLUMN on all versions; skip for portability.
        // On MySQL/Postgres you can uncomment:
        // manager.alter_table(Table::alter().table(Alias::new("roles")).drop_column(Alias::new("guard_name")).to_owned()).await?;
        // manager.alter_table(Table::alter().table(Alias::new("settings")).drop_column(Alias::new("favicon")).to_owned()).await?;
        let _ = manager;
        Ok(())
    }
}
