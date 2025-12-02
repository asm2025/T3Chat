// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        id -> Uuid,
        action_id -> Text,
        user_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        action_type -> Nullable<Text>,
        domain -> Nullable<Text>,
        endpoint_url -> Nullable<Text>,
        settings -> Nullable<Jsonb>,
        auth_type -> Nullable<Text>,
        auth_config -> Nullable<Jsonb>,
        openapi_spec -> Nullable<Text>,
        privacy_policy_url -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    agent_actions (agent_id, action_id) {
        agent_id -> Uuid,
        action_id -> Uuid,
        is_enabled -> Nullable<Bool>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agent_conversation_starters (id) {
        id -> Uuid,
        agent_id -> Uuid,
        text -> Text,
        order_index -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agent_hierarchy (parent_agent_id, sub_agent_id) {
        parent_agent_id -> Uuid,
        sub_agent_id -> Uuid,
        order_index -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agent_tools (agent_id, tool_id) {
        agent_id -> Uuid,
        tool_id -> Uuid,
        configuration -> Nullable<Jsonb>,
        is_enabled -> Nullable<Bool>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agents (id) {
        id -> Uuid,
        agent_id -> Text,
        author_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        instructions -> Nullable<Text>,
        avatar_filepath -> Nullable<Text>,
        avatar_source -> Nullable<Text>,
        provider -> Text,
        model -> Text,
        model_parameters -> Nullable<Jsonb>,
        access_level -> Nullable<Int4>,
        recursion_limit -> Nullable<Int4>,
        hide_sequential_outputs -> Nullable<Bool>,
        end_after_tools -> Nullable<Bool>,
        is_collaborative -> Nullable<Bool>,
        tool_resources -> Nullable<Jsonb>,
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
        max_output_tokens -> Nullable<Int4>,
        supports_streaming -> Nullable<Bool>,
        supports_images -> Nullable<Bool>,
        supports_functions -> Nullable<Bool>,
        supports_vision -> Nullable<Bool>,
        cost_per_input_token -> Nullable<Numeric>,
        cost_per_output_token -> Nullable<Numeric>,
        is_active -> Nullable<Bool>,
        deprecated_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        provider_id -> Nullable<Uuid>,
        disabled -> Bool,
        is_paid -> Bool,
    }
}

diesel::table! {
    ai_providers (id) {
        id -> Uuid,
        provider_id -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        base_url -> Nullable<Text>,
        website_url -> Nullable<Text>,
        documentation_url -> Nullable<Text>,
        disabled -> Bool,
        is_active -> Bool,
        requires_api_key -> Bool,
        supports_streaming -> Bool,
        supports_images -> Bool,
        supports_functions -> Bool,
        supports_vision -> Bool,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    assistant_conversation_starters (id) {
        id -> Uuid,
        assistant_id -> Uuid,
        text -> Text,
        order_index -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    assistant_tools (assistant_id, tool_id) {
        assistant_id -> Uuid,
        tool_id -> Uuid,
        configuration -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    assistants (id) {
        id -> Uuid,
        assistant_id -> Text,
        user_id -> Text,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        instructions -> Nullable<Text>,
        avatar_filepath -> Nullable<Text>,
        avatar_source -> Nullable<Text>,
        model -> Text,
        tools -> Nullable<Jsonb>,
        file_ids -> Nullable<Array<Nullable<Uuid>>>,
        access_level -> Nullable<Int4>,
        append_current_datetime -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    balances (id) {
        id -> Uuid,
        user_id -> Text,
        token_credit_balance -> Nullable<Int8>,
        token_credit_consumed -> Nullable<Int8>,
        monetary_balance -> Nullable<Numeric>,
        monetary_consumed -> Nullable<Numeric>,
        currency -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    conversation_tags_map (conversation_id, tag_id) {
        conversation_id -> Uuid,
        tag_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    conversations (id) {
        id -> Uuid,
        conversation_id -> Text,
        user_id -> Text,
        title -> Nullable<Text>,
        endpoint -> Text,
        model -> Text,
        model_label -> Nullable<Text>,
        model_parameters -> Nullable<Jsonb>,
        system_message -> Nullable<Text>,
        instructions -> Nullable<Text>,
        feature_flags -> Nullable<Jsonb>,
        agent_id -> Nullable<Uuid>,
        assistant_id -> Nullable<Uuid>,
        agent_options -> Nullable<Jsonb>,
        is_archived -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        file_id -> Text,
        user_id -> Text,
        conversation_id -> Nullable<Uuid>,
        filename -> Text,
        filepath -> Text,
        mime_type -> Text,
        size_bytes -> Int8,
        file_type -> Text,
        text_content -> Nullable<Text>,
        is_embedded -> Nullable<Bool>,
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
        source -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        usage_count -> Nullable<Int4>,
        last_used_at -> Nullable<Timestamptz>,
        is_temporary -> Nullable<Bool>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        message_id -> Text,
        conversation_id -> Uuid,
        parent_message_id -> Nullable<Uuid>,
        role -> Text,
        text -> Nullable<Text>,
        is_created_by_user -> Bool,
        model -> Nullable<Text>,
        endpoint -> Nullable<Text>,
        content -> Nullable<Jsonb>,
        token_count -> Nullable<Int4>,
        finish_reason -> Nullable<Text>,
        error -> Nullable<Bool>,
        file_ids -> Nullable<Array<Nullable<Uuid>>>,
        tool_call_id -> Nullable<Text>,
        plugin_data -> Nullable<Jsonb>,
        thread_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    presets (id) {
        id -> Uuid,
        preset_id -> Text,
        user_id -> Text,
        title -> Text,
        is_default -> Nullable<Bool>,
        order_index -> Nullable<Int4>,
        endpoint -> Text,
        model -> Text,
        model_label -> Nullable<Text>,
        model_parameters -> Nullable<Jsonb>,
        system_message -> Nullable<Text>,
        instructions -> Nullable<Text>,
        feature_flags -> Nullable<Jsonb>,
        agent_id -> Nullable<Uuid>,
        agent_options -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    project_agents (project_id, agent_id) {
        project_id -> Uuid,
        agent_id -> Uuid,
        role -> Nullable<Text>,
        order_index -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    projects (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        owner_id -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    prompt_groups (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        author_id -> Text,
        project_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    prompts (id) {
        id -> Uuid,
        group_id -> Uuid,
        title -> Nullable<Text>,
        prompt_text -> Text,
        prompt_type -> Text,
        variables -> Nullable<Array<Nullable<Text>>>,
        order_index -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    roles (name) {
        name -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    shared_links (id) {
        id -> Uuid,
        share_id -> Text,
        conversation_id -> Uuid,
        user_id -> Text,
        is_public -> Nullable<Bool>,
        is_anonymous -> Nullable<Bool>,
        title -> Nullable<Text>,
        password_hash -> Nullable<Text>,
        max_views -> Nullable<Int4>,
        view_count -> Nullable<Int4>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_viewed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tags (id) {
        id -> Uuid,
        user_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        color -> Nullable<Text>,
        position -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    tool_calls (id) {
        id -> Uuid,
        message_id -> Uuid,
        tool_call_id -> Text,
        tool_name -> Text,
        tool_type -> Nullable<Text>,
        arguments -> Nullable<Jsonb>,
        result -> Nullable<Jsonb>,
        status -> Nullable<Text>,
        error_message -> Nullable<Text>,
        output_file_ids -> Nullable<Array<Nullable<Uuid>>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    tools (id) {
        id -> Uuid,
        name -> Text,
        display_name -> Text,
        description -> Nullable<Text>,
        tool_type -> Text,
        icon_url -> Nullable<Text>,
        is_active -> Nullable<Bool>,
        is_system -> Nullable<Bool>,
        configuration_schema -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        user_id -> Text,
        message_id -> Nullable<Uuid>,
        conversation_id -> Nullable<Uuid>,
        provider -> Text,
        model -> Text,
        input_tokens -> Nullable<Int4>,
        output_tokens -> Nullable<Int4>,
        total_tokens -> Int4,
        cost_per_input_token -> Nullable<Numeric>,
        cost_per_output_token -> Nullable<Numeric>,
        total_cost -> Nullable<Numeric>,
        currency -> Nullable<Text>,
        transaction_type -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_api_keys (id) {
        id -> Uuid,
        user_id -> Text,
        provider -> Text,
        encrypted_key -> Text,
        key_name -> Nullable<Text>,
        is_default -> Nullable<Bool>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_used_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_features (id) {
        id -> Uuid,
        user_id -> Text,
        feature -> Text,
        enabled -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_roles (user_id, role_name) {
        user_id -> Text,
        role_name -> Text,
        assigned_at -> Timestamptz,
        assigned_by -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        email_verified -> Nullable<Bool>,
        name -> Nullable<Text>,
        username -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
        provider -> Text,
        role -> Nullable<Text>,
        password_hash -> Nullable<Text>,
        two_factor_enabled -> Nullable<Bool>,
        totp_secret -> Nullable<Text>,
        preferences -> Nullable<Jsonb>,
        terms_accepted -> Nullable<Bool>,
        terms_accepted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        normalized_email -> Text,
        normalized_username -> Nullable<Text>,
        disabled -> Bool,
        locked_out -> Bool,
        lockout_end -> Nullable<Timestamptz>,
        access_failed_count -> Int4,
        password_changed_at -> Nullable<Timestamptz>,
        last_login_at -> Nullable<Timestamptz>,
        login_count -> Int4,
    }
}

diesel::joinable!(actions -> users (user_id));
diesel::joinable!(agent_actions -> actions (action_id));
diesel::joinable!(agent_actions -> agents (agent_id));
diesel::joinable!(agent_conversation_starters -> agents (agent_id));
diesel::joinable!(agent_tools -> agents (agent_id));
diesel::joinable!(agent_tools -> tools (tool_id));
diesel::joinable!(agents -> users (author_id));
diesel::joinable!(ai_models -> ai_providers (provider_id));
diesel::joinable!(assistant_conversation_starters -> assistants (assistant_id));
diesel::joinable!(assistant_tools -> assistants (assistant_id));
diesel::joinable!(assistant_tools -> tools (tool_id));
diesel::joinable!(assistants -> users (user_id));
diesel::joinable!(balances -> users (user_id));
diesel::joinable!(conversation_tags_map -> conversations (conversation_id));
diesel::joinable!(conversation_tags_map -> tags (tag_id));
diesel::joinable!(conversations -> agents (agent_id));
diesel::joinable!(conversations -> assistants (assistant_id));
diesel::joinable!(conversations -> users (user_id));
diesel::joinable!(files -> conversations (conversation_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(presets -> agents (agent_id));
diesel::joinable!(presets -> users (user_id));
diesel::joinable!(project_agents -> agents (agent_id));
diesel::joinable!(project_agents -> projects (project_id));
diesel::joinable!(projects -> users (owner_id));
diesel::joinable!(prompt_groups -> projects (project_id));
diesel::joinable!(prompt_groups -> users (author_id));
diesel::joinable!(prompts -> prompt_groups (group_id));
diesel::joinable!(shared_links -> conversations (conversation_id));
diesel::joinable!(shared_links -> users (user_id));
diesel::joinable!(tags -> users (user_id));
diesel::joinable!(tool_calls -> messages (message_id));
diesel::joinable!(transactions -> conversations (conversation_id));
diesel::joinable!(transactions -> messages (message_id));
diesel::joinable!(transactions -> users (user_id));
diesel::joinable!(user_api_keys -> users (user_id));
diesel::joinable!(user_features -> users (user_id));
diesel::joinable!(user_roles -> roles (role_name));

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    agent_actions,
    agent_conversation_starters,
    agent_hierarchy,
    agent_tools,
    agents,
    ai_models,
    ai_providers,
    assistant_conversation_starters,
    assistant_tools,
    assistants,
    balances,
    conversation_tags_map,
    conversations,
    files,
    messages,
    presets,
    project_agents,
    projects,
    prompt_groups,
    prompts,
    roles,
    shared_links,
    tags,
    tool_calls,
    tools,
    transactions,
    user_api_keys,
    user_features,
    user_roles,
    users,
);
