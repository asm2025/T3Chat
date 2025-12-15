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
    presets?: boolean;
    // T3Chat specific legacy fallbacks if needed, or purely LibreChat style
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
    provider: string;
    model: string;
    description?: string | null;
    icon?: string | null;
    default?: boolean;
    parameters?: unknown;
}

export interface StartupNotice {
    kind: StartupNoticeKind;
    message: string;
}

export type StartupNoticeKind = "info" | "warning";
