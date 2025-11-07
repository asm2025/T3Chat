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
                    .table(AiModels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiModels::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AiModels::Provider).string().not_null())
                    .col(ColumnDef::new(AiModels::ModelId).string().not_null())
                    .col(ColumnDef::new(AiModels::DisplayName).string().not_null())
                    .col(ColumnDef::new(AiModels::Description).text())
                    .col(ColumnDef::new(AiModels::ContextWindow).integer().not_null())
                    .col(
                        ColumnDef::new(AiModels::SupportsStreaming)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AiModels::SupportsImages)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AiModels::SupportsFunctions)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(AiModels::CostPerToken).decimal())
                    .col(
                        ColumnDef::new(AiModels::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AiModels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AiModels::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-ai_models-provider-model_id")
                    .if_not_exists()
                    .table(AiModels::Table)
                    .col(AiModels::Provider)
                    .col(AiModels::ModelId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-ai_models-is_active")
                    .if_not_exists()
                    .table(AiModels::Table)
                    .col(AiModels::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiModels::Table).to_owned())
            .await?;

        Ok(())
    }
}

