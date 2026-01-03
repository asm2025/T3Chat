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
        crate::api::health::health_check,
        crate::api::models::list_models,
        crate::api::models::list_all_models,
        crate::api::models::get_model,
        crate::api::models::list_my_models,
        crate::api::models::enable_model,
        crate::api::models::disable_model,
        crate::api::chats::list_chats,
        crate::api::chats::create_chat,
        crate::api::chats::get_chat,
        crate::api::chats::update_chat,
        crate::api::chats::delete_chat,
        crate::api::chats::messages::get_messages,
        crate::api::chats::messages::create_message,
        crate::api::chats::messages::update_message,
        crate::api::chats::messages::delete_message,
        crate::api::chats::messages::clear_messages,
        crate::api::chat::chat,
        crate::api::chat::stream_chat,
        crate::api::user::profile,
        crate::api::user::update_profile,
        crate::api::user_api_keys::list_keys,
        crate::api::user_api_keys::create_key,
        crate::api::user_api_keys::delete_key,
        crate::api::features::list_features,
        crate::api::features::update_feature,
        crate::api::presets::list_presets,
        crate::api::presets::get_preset,
        crate::api::presets::create_preset,
        crate::api::presets::update_preset,
        crate::api::presets::delete_preset,
        crate::api::presets::reorder_presets
    ),
    components(
        schemas(
            crate::api::health::HealthResponse,
            crate::api::models::ModelResponse,
            crate::api::common::UserResponse,
            crate::api::chats::ChatResponse,
            crate::api::chats::MessageResponse,
            crate::api::chats::ChatWithMessagesResponse,
            crate::api::chats::ChatListResponse,
            crate::api::chats::CreateChatRequest,
            crate::api::chats::UpdateChatRequest,
            crate::api::chats::messages::CreateMessageRequest,
            crate::api::chats::messages::UpdateMessageRequest,
            crate::api::chat::ChatRequest,
            crate::api::chat::ChatCompletionResponse,
            crate::api::user::UpdateUserRequest,
            crate::api::user_api_keys::UserApiKeyResponse,
            crate::api::user_api_keys::CreateUserApiKeyRequest,
            crate::api::features::UserFeatureResponse,
            crate::api::features::UserFeaturesResponse,
            crate::api::features::UpdateFeatureRequest,
            crate::db::models::Preset,
            crate::api::presets::CreatePresetRequest,
            crate::api::presets::UpdatePresetRequest,
            crate::api::presets::ReorderPresetsRequest
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Models", description = "AI model catalogue"),
        (name = "Chats", description = "Chat management"),
        (name = "Messages", description = "Chat message management"),
        (name = "Chat", description = "Chat completion endpoints"),
        (name = "User", description = "Authenticated user profile"),
        (name = "User API Keys", description = "API key management"),
        (name = "Features", description = "User feature preferences"),
        (name = "Presets", description = "Chat preset management")
    ),
    modifiers(&BearerAuthAddon)
)]
pub struct ApiDoc;
