use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(Default)]
struct BearerAuthAddon;

impl Modify for BearerAuthAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::v1::health::health_check,
        crate::api::v1::models::list_models,
        crate::api::v1::models::get_model,
        crate::api::v1::chats::list_chats,
        crate::api::v1::chats::create_chat,
        crate::api::v1::chats::get_chat,
        crate::api::v1::chats::update_chat,
        crate::api::v1::chats::delete_chat,
        crate::api::v1::chats::messages::get_messages,
        crate::api::v1::chats::messages::create_message,
        crate::api::v1::chat::chat,
        crate::api::v1::chat::stream_chat,
        crate::api::v1::user::profile,
        crate::api::v1::user::update_profile,
        crate::api::v1::user_api_keys::list_keys,
        crate::api::v1::user_api_keys::create_key,
        crate::api::v1::user_api_keys::delete_key
    ),
    components(
        schemas(
            crate::api::v1::health::HealthResponse,
            crate::api::v1::models::ModelResponse,
            crate::api::common::UserResponse,
            crate::api::v1::chats::ChatResponse,
            crate::api::v1::chats::MessageResponse,
            crate::api::v1::chats::ChatWithMessagesResponse,
            crate::api::v1::chats::ChatListResponse,
            crate::api::v1::chats::CreateChatRequest,
            crate::api::v1::chats::UpdateChatRequest,
            crate::api::v1::chats::messages::CreateMessageRequest,
            crate::api::v1::chat::ChatRequest,
            crate::api::v1::chat::ChatCompletionResponse,
            crate::api::v1::user::UpdateUserRequest,
            crate::api::v1::user_api_keys::UserApiKeyResponse,
            crate::api::v1::user_api_keys::CreateUserApiKeyRequest
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Models", description = "AI model catalogue"),
        (name = "Chats", description = "Chat management"),
        (name = "Messages", description = "Chat message management"),
        (name = "Chat", description = "Chat completion endpoints"),
        (name = "User", description = "Authenticated user profile"),
        (name = "User API Keys", description = "API key management")
    ),
    modifiers(&BearerAuthAddon)
)]
pub struct ApiDoc;
