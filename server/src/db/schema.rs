// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Text,
        email -> Nullable<Text>,
        display_name -> Nullable<Text>,
        image_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ai_models (id) {
        id -> Uuid,
        provider -> Text,
        model_id -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        context_window -> Int4,
        supports_streaming -> Bool,
        supports_images -> Bool,
        supports_functions -> Bool,
        cost_per_token -> Nullable<Numeric>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_api_keys (id) {
        id -> Uuid,
        user_id -> Text,
        provider -> Text,
        encrypted_key -> Text,
        is_default -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    chats (id) {
        id -> Uuid,
        user_id -> Text,
        title -> Text,
        model_provider -> Text,
        model_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        chat_id -> Uuid,
        role -> Text,
        content -> Text,
        metadata -> Nullable<Jsonb>,
        parent_message_id -> Nullable<Uuid>,
        sequence_number -> Int4,
        created_at -> Timestamptz,
        tokens_used -> Nullable<Int4>,
        model_used -> Nullable<Text>,
    }
}

diesel::joinable!(chats -> users (user_id));
diesel::joinable!(messages -> chats (chat_id));
diesel::joinable!(user_api_keys -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, ai_models, user_api_keys, chats, messages,);
