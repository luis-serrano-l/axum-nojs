use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "tasks",
            &[
                ("id", ColType::PkAuto),
                ("title", ColType::String),
                ("done", ColType::Boolean),
                ("due_on", ColType::DateNull),
                ("starts_at", ColType::DateTimeNull),
                ("remind_at", ColType::TimestampWithTimeZoneNull),
                ("price", ColType::DecimalNull),
                ("status", ColType::String),
                ("size", ColType::BigIntegerNull),
            ],
            &[("user", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "tasks").await
    }
}
