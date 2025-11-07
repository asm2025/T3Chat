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
                    .table(UserApiKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserApiKeys::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserApiKeys::UserId).string().not_null())
                    .col(ColumnDef::new(UserApiKeys::Provider).string().not_null())
                    .col(ColumnDef::new(UserApiKeys::EncryptedKey).text().not_null())
                    .col(
                        ColumnDef::new(UserApiKeys::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserApiKeys::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserApiKeys::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_api_keys-user_id")
                            .from(UserApiKeys::Table, UserApiKeys::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-user_api_keys-user_id-provider")
                    .if_not_exists()
                    .table(UserApiKeys::Table)
                    .col(UserApiKeys::UserId)
                    .col(UserApiKeys::Provider)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-user_api_keys-is_default")
                    .if_not_exists()
                    .table(UserApiKeys::Table)
                    .col(UserApiKeys::IsDefault)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserApiKeys::Table).to_owned())
            .await?;

        Ok(())
    }
}

