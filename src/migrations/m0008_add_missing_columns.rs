//! Adds columns missing from initial schema vs NodeAdmin standard:
//! - `roles.guard_name` (VARCHAR 20, default 'web')
//! - `settings.favicon` (VARCHAR 255, nullable)

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // guard_name to roles — ignore if already exists (SQLite IF NOT EXISTS compat)
        let has_guard: i64 = db
            .query_one(sea_orm::Statement::from_string(
                manager.get_database_backend(),
                "SELECT COUNT(*) FROM pragma_table_info('roles') WHERE name='guard_name'".to_owned(),
            ))
            .await?
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .unwrap_or(0);
        if has_guard == 0 {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("roles"))
                        .add_column(
                            ColumnDef::new(Alias::new("guard_name"))
                                .string_len(20)
                                .null()
                                .default("web"),
                        )
                        .to_owned(),
                )
                .await?;
        }

        // favicon to settings — ignore if already exists
        let has_favicon: i64 = db
            .query_one(sea_orm::Statement::from_string(
                manager.get_database_backend(),
                "SELECT COUNT(*) FROM pragma_table_info('settings') WHERE name='favicon'".to_owned(),
            ))
            .await?
            .and_then(|r| r.try_get_by_index::<i64>(0).ok())
            .unwrap_or(0);
        if has_favicon == 0 {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("settings"))
                        .add_column(ColumnDef::new(Alias::new("favicon")).string().null())
                        .to_owned(),
                )
                .await?;
        }

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
