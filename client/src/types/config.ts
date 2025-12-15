export interface StartupConfigResponse {
    appTitle?: string | null;
    interface?: InterfaceConfig;
    providers: ProviderSummary[];
    modelSpecs: ModelSpec[];
    notices?: StartupNotice[];
}

export interface InterfaceConfig {
    endpointsMenu?: boolean;
    modelSelect?: boolean;
    parametersMenu?: boolean;
    sidePanel?: boolean;
    presets?: boolean;
    // T3Chat specific: default model selection
    defaultModelSpec?: string;
    defaultProvider?: string;
    defaultModel?: string;
}

export interface ProviderSummary {
    key: string;
    label: string;
    icon?: string | null;
    description?: string | null;
}

export interface ModelSpec {
    name: string;
    label: string;
    description?: string | null;
    icon?: string | null;
    default?: boolean;
    preset: {
        endpoint: string;
        model: string;
        model_label?: string | null;
        greeting?: string | null;
        prompt_prefix?: string | null;
        temperature?: number | null;
        top_p?: number | null;
        presence_penalty?: number | null;
        frequency_penalty?: number | null;
        resend_files?: boolean;
        image_detail?: string | null;
        tools?: boolean;
    };
}

export interface StartupNotice {
    kind: StartupNoticeKind;
    message: string;
}

export type StartupNoticeKind = "info" | "warning";
