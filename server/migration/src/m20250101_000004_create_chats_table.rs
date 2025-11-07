use sea_orm_migration::prelude::*;

use crate::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Chats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Chats::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Chats::UserId).string().not_null())
                    .col(ColumnDef::new(Chats::Title).string().not_null())
                    .col(ColumnDef::new(Chats::ModelProvider).string().not_null())
                    .col(ColumnDef::new(Chats::ModelId).string().not_null())
                    .col(
                        ColumnDef::new(Chats::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Chats::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Chats::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chats-user_id")
                            .from(Chats::Table, Chats::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-chats-user_id")
                    .if_not_exists()
                    .table(Chats::Table)
                    .col(Chats::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-chats-deleted_at")
                    .if_not_exists()
                    .table(Chats::Table)
                    .col(Chats::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-chats-updated_at")
                    .if_not_exists()
                    .table(Chats::Table)
                    .col(Chats::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chats::Table).to_owned())
            .await?;

        Ok(())
    }
}

