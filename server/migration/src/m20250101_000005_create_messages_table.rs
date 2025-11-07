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
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Messages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Messages::ChatId).uuid().not_null())
                    .col(ColumnDef::new(Messages::Role).string().not_null())
                    .col(ColumnDef::new(Messages::Content).text().not_null())
                    .col(ColumnDef::new(Messages::Metadata).json_binary())
                    .col(ColumnDef::new(Messages::ParentMessageId).uuid())
                    .col(ColumnDef::new(Messages::SequenceNumber).integer().not_null())
                    .col(
                        ColumnDef::new(Messages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Messages::TokensUsed).integer())
                    .col(ColumnDef::new(Messages::ModelUsed).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-messages-chat_id")
                            .from(Messages::Table, Messages::ChatId)
                            .to(Chats::Table, Chats::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-messages-parent_message_id")
                            .from(Messages::Table, Messages::ParentMessageId)
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-messages-chat_id-sequence")
                    .if_not_exists()
                    .table(Messages::Table)
                    .col(Messages::ChatId)
                    .col(Messages::SequenceNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-messages-parent_message_id")
                    .if_not_exists()
                    .table(Messages::Table)
                    .col(Messages::ParentMessageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await?;

        Ok(())
    }
}

