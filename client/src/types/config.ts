export interface StartupConfigResponse {
    app_title?: string | null;
    interface: InterfaceConfigResponse;
    providers: ProviderSummaryResponse[];
    model_specs: ModelSpecResponse[];
    notices?: StartupNotice[];
}

export interface InterfaceConfigResponse {
    model_select_enabled: boolean;
    default_model_spec?: string | null;
    default_provider?: string | null;
    default_model?: string | null;
}

export interface ProviderSummaryResponse {
    key: string;
    label: string;
    icon?: string | null;
}

export interface ModelSpecResponse {
    name: string;
    label: string;
    provider: string;
    model: string;
    description?: string | null;
    icon?: string | null;
    default: boolean;
    parameters?: unknown;
}

export interface StartupNotice {
    kind: StartupNoticeKind;
    message: string;
}

export type StartupNoticeKind = "info" | "warning";

