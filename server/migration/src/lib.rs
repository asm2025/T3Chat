pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_users_table;
mod m20250101_000002_create_ai_models_table;
mod m20250101_000003_create_user_api_keys_table;
mod m20250101_000004_create_chats_table;
mod m20250101_000005_create_messages_table;

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

#[derive(DeriveIden)]
pub enum AiModels {
    Table,
    Id,
    Provider,
    ModelId,
    DisplayName,
    Description,
    ContextWindow,
    SupportsStreaming,
    SupportsImages,
    SupportsFunctions,
    CostPerToken,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum UserApiKeys {
    Table,
    Id,
    UserId,
    Provider,
    EncryptedKey,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Chats {
    Table,
    Id,
    UserId,
    Title,
    ModelProvider,
    ModelId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
pub enum Messages {
    Table,
    Id,
    ChatId,
    Role,
    Content,
    Metadata,
    ParentMessageId,
    SequenceNumber,
    CreatedAt,
    TokensUsed,
    ModelUsed,
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_users_table::Migration),
            Box::new(m20250101_000002_create_ai_models_table::Migration),
            Box::new(m20250101_000003_create_user_api_keys_table::Migration),
            Box::new(m20250101_000004_create_chats_table::Migration),
            Box::new(m20250101_000005_create_messages_table::Migration),
        ]
    }
}
