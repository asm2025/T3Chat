## T3Chat Models & Providers – Developer Implementation Plan

This document turns `!ref/plan.models.md` into a **developer‑focused** plan with concrete files, steps, and sequencing. Work is split between a **Backend developer** and a **UI/Frontend developer** and can be done largely in parallel.

---

## Roles & Ownership

- **Backend developer**
  - Owns tasks **B1–B13** from `plan.models.md`.
  - Delivers YAML config loading, provider registry, model catalog, DB alignment, and `/config/startup` + `/models` API surfaces (plus OpenRouter/RouteLLM backends).

- **UI developer**
  - Owns tasks **F1–F8** from `plan.models.md`.
  - Delivers TypeScript types, config/models client + hooks, `ModelSelector` UX, and Models/Chat page integration (plus provider‑specific UI polish).

Suggested timeline:

- **Week 1–2**: Backend Track A/B, Frontend Track D (types, client, hooks).
- **Week 2–3**: Frontend Track E (ModelSelector + pages), Backend Track C (DB + validation).
- **Week 3–4**: Integration, end‑to‑end testing, polish, and OpenRouter/RouteLLM wiring.

---

## Backend Developer Plan

### 1. Config Schema & Loader (`t3chat.yaml`) – Tasks B1–B3

- **Goal**: Introduce a single YAML config (`t3chat.yaml`) that defines interface flags, providers, models, and model specs. Provide a robust loader with env interpolation and an example file.

- **Files to add**
  - **Config module**
    - `server/src/config/mod.rs`
    - `server/src/config/t3chat.rs`
  - **YAML config files**
    - `t3chat.yaml` (runtime, not committed; read via env var).
    - `t3chat.example.yaml` (committed, documented example).

- **Implementation steps**
  1. **Define Rust config structs (Task B1)**
     - In `server/src/config/t3chat.rs`, define:
       - `T3ChatConfig` with:
         - `interface: InterfaceConfig` (e.g. `model_select_enabled`, `default_model_spec`, etc.).
         - `providers: HashMap<String, ProviderConfig>` or explicit struct with fields `openai`, `anthropic`, `google`, `bedrock`, `azure`, `openrouter`, `routellm`, `custom: Vec<CustomProviderConfig>`.
         - `model_specs: Vec<ModelSpecConfig>`.
       - `ProviderConfig` fields:
         - `api_key: Option<String>` (may contain `${ENV_VAR}`).
         - `base_url: Option<String>`.
         - `models: ProviderModelsConfig` with `default: Vec<String>`, `fetch: bool`, `user_id_query: Option<bool>`, etc.
       - `ModelSpecConfig` fields:
         - `{ name, label, endpoint, model, parameters (serde_json::Value or typed), description?, icon?, default? }`.
     - Derive `Deserialize`, `Clone`, `Debug` as appropriate.

  2. **Implement loader with env interpolation (Task B2)**
     - In `server/src/config/mod.rs`:
       - Expose a `pub fn load_config() -> Result<T3ChatConfig, ConfigError>`.
       - Determine config path:
         - Read `T3CHAT_CONFIG` env var.
         - Fallback to `t3chat.yaml` in the project root (or `server/` root, but keep this consistent and documented).
       - Use `serde_yaml` to parse the file.
       - Implement a small helper to walk the deserialized structure and perform `${VAR}` interpolation for any string fields:
         - This can be done before or after deserialization (e.g. read file as string, replace `${VAR}` with `std::env::var("VAR")`).
       - Define `ConfigError` enum for:
         - Missing file (if treated as fatal) vs allowed fallback.
         - YAML parse errors.
         - Missing required keys or malformed content (validation step).

  3. **Add basic validation (Task B1/B2)**
     - After loading, validate:
       - Provider keys use expected names (warn or ignore unknown).
       - If `models.fetch == false`, ensure `models.default` is non‑empty for that provider.
       - Model specs reference existing `provider` keys and non‑empty `model`.
     - Decide on behavior:
       - Fatal vs warning per error type; log clearly using existing logging utilities.

  4. **Create example configuration (Task B3)**
     - Add `t3chat.example.yaml` at repo root with:
       - Intro comment explaining how to copy to `t3chat.yaml`.
       - Sections:
         - `interface`: flags for model selection, default provider/model/spec.
         - `providers`:
           - Examples for `openai`, `anthropic`, `google`, `bedrock`, `azure`.
           - `openrouter` with `${OPENROUTER_API_KEY}` and `https://openrouter.ai/api/v1`.
           - `routellm` stub with a `routes` list.
           - At least one `custom` provider example.
         - `model_specs`: examples mapping to main providers (e.g. “GPT‑4o Default”, “Claude 3 Sonnet Coding”).
       - Make sure comments annotate each field and, where relevant, mention required env vars to add to `.env`.

  5. **Wire loader into server startup**
     - In `server/src/main.rs`:
       - During app initialization, call `config::load_config()` and:
         - Store result in `AppState` (e.g. `pub t3_config: Arc<T3ChatConfig>`).
         - Decide if startup should fail when config can’t be loaded vs run with safe defaults.

### 2. Provider Registry & Model Catalog – Tasks B4–B6, B10–B11

- **Goal**: Provide a normalized provider abstraction and a cached model catalog that merges config defaults with any provider‑discovered models. Extend to OpenRouter now, design RouteLLM for later.

- **Files to add/update**
  - **Provider registry**
    - `server/src/ai/providers/mod.rs` (already exists; extend).
    - `server/src/ai/providers/openrouter.rs` (new).
    - `server/src/ai/providers/routellm.rs` (placeholder for later).
  - **Model catalog**
    - `server/src/ai/model_catalog.rs` (new helper module).
  - **Types**
    - `server/src/ai/types.rs` (already exists; add `ModelInfo`, capability flags if not present).

- **Implementation steps**
  1. **Define provider registry trait (Task B4)**
     - In `server/src/ai/providers/mod.rs`, define:
       - `pub type ProviderKey = String;` or a small enum (e.g. `OpenAI`, `Anthropic`, etc.) plus conversions to/from string.
       - A trait `ModelListingProvider` with:
         - `async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;`
         - `fn key(&self) -> ProviderKey;`
         - Optional capability helpers (even if stubbed).
       - Reuse or extend existing provider client types (OpenAI, Anthropic, Google) to implement this trait; add integration glue as needed.

  2. **Implement dynamic model discovery (Task B5)**
     - For providers that support listing:
       - **OpenAI**: call the models/list endpoint or use a curated static list if listing is not practical.
       - **OpenRouter**: implement `openrouter.rs` client:
         - Base URL `https://openrouter.ai/api/v1`.
         - Auth header per spec (e.g. `Authorization: Bearer <key>`).
         - Implement `list_models()` against `/models` and map response into `ModelInfo`.
       - **Others**: stub `list_models()` to return an empty list or static defaults (to be refined later).

  3. **Create `ModelCatalog` service (Task B6)**
     - Add `server/src/ai/model_catalog.rs` with:
       - `pub struct ModelCatalog { /* provider map, config reference, cache */ }`.
       - `pub async fn build(config: Arc<T3ChatConfig>, providers: Vec<Arc<dyn ModelListingProvider>>)` which:
         - Iterates each configured provider.
         - If `models.fetch == true`:
           - Calls provider’s `list_models()`; falls back to `models.default` if empty or on error.
         - If `models.fetch == false`:
           - Uses `models.default` as the canonical list.
         - Normalizes all into `HashMap<ProviderKey, Vec<ModelInfo>>`.
       - Provide query methods:
         - `pub fn all(&self) -> &HashMap<ProviderKey, Vec<ModelInfo>>;`
         - `pub fn for_provider(&self, key: &ProviderKey) -> Option<&[ModelInfo]>;`
         - `pub fn find(&self, key: &ProviderKey, model_id: &str) -> Option<&ModelInfo>;`.
     - Store `ModelCatalog` in `AppState` as `Arc<ModelCatalog>`.

  4. **OpenRouter provider integration (Task B10)**
     - Implement `server/src/ai/providers/openrouter.rs`:
       - Struct `OpenRouterProvider { client, api_key, base_url }`.
       - Implement `ModelListingProvider` trait and any other shared traits used by your AI manager.
       - Use config values from `T3ChatConfig.providers["openrouter"]`.
     - Register this provider in whatever factory or manager builds providers at startup.

  5. **RouteLLM / Abacus.ai design (Task B11)**
     - For this iteration, focus on **research + config shape**, minimal code:
       - Define `RouteLLMProviderConfig` in `t3chat.rs`:
         - `api_key`, `base_url`, `routes: ProviderModelsConfig` (same structure as models).
       - Document in `t3chat.example.yaml` how `routes` map to logical “models” (e.g. `route/general`, `route/coding`).
       - Add `routellm` to provider key enum/strings without implementing its client yet.
     - Optionally stub `providers/routellm.rs` with `todo!()` and keep it disabled in runtime wiring until design is finalized.

### 3. DB Integration & Validation – Tasks B7–B9

- **Goal**: Align `ai_provider`/`ai_model` tables with config and enforce that only configured+enabled models can be used in chat/preset/agent flows.

- **Files to update**
  - `server/src/db/models/ai_provider.rs`
  - `server/src/db/models/ai_model.rs`
  - Relevant Diesel migrations under `server/migrations/` for provider/model metadata.
  - `server/src/api/v1/chat/*.rs`, `server/src/api/v1/chats/*.rs`, `server/src/api/v1/admin/*.rs` (wherever `{provider, model}` are accepted).

- **Implementation steps**
  1. **Schema review & migration design (Task B7)**
     - Inspect existing `ai_provider` and `ai_model` tables and models to identify missing fields required by config:
       - Provider type/name, base URL, “source” (`yaml`, `dynamic`, `built‑in`), capabilities, etc.
       - For models: `family`, `is_deprecated`, `is_default`, capability flags, any pricing hints (optional).
     - Decide on strategy:
       - Either treat DB as **analytics/permissions mirror** of config (sync on startup) or keep DB as primary and mark config as a filter.
       - For now, favor a **config‑primary** approach with optional DB mirroring.
     - Add migrations as needed to support:
       - `config_source` enum/string field.
       - Extra metadata columns that are useful downstream.

  2. **Config‑to‑DB sync (Task B8)**
     - Implement a sync routine, e.g. in `server/src/ai/model_catalog.rs` or a new `server/src/db/sync.rs`:
       - On startup, for each provider/model pair from `ModelCatalog`:
         - Upsert into `ai_provider` and `ai_model` tables with `config_source = "yaml"` (or similar).
         - For models missing from config but present in DB:
           - Mark as disabled/deprecated instead of deleting, to avoid breaking existing data.
       - Ensure idempotency so repeated startups don’t create duplicates.
       - Log summary (counts of added/updated/disabled providers/models).

  3. **Enforce config in chat flows (Task B9)**
     - In chat creation/update and message/agent APIs (e.g. `server/src/api/v1/chat`, `server/src/api/v1/chats`, `server/src/api/v1/admin`):
       - When a `{provider, model}` is requested:
         - First validate against `ModelCatalog`:
           - Check provider exists and model is present.
         - Optionally cross‑check DB (e.g. `ai_model.is_active`).
       - Return:
         - 400/422 with a clear error message if a model is not configured or has been disabled.
       - If there is a pre‑existing default model logic, update it to prefer config defaults (`interface.default_model_spec` or `interface.default_provider/model`).

### 4. Startup Config & Models API – Tasks B12–B13

- **Goal**: Expose non‑sensitive configuration and normalized model lists for the UI to consume.

- **Files to add/update**
  - `server/src/api/v1/config/mod.rs` (new).
  - `server/src/api/v1/config/startup.rs` (new).
  - `server/src/api/v1/models/mod.rs` (already exists; may need to extend or add a new route).
  - `server/src/main.rs` routing table (to add `/api/v1/config/startup` and any new models route).

- **Implementation steps**
  1. **Startup config endpoint (Task B12)**
     - Add `server/src/api/v1/config/startup.rs`:
       - Handler `GET /api/v1/config/startup` that:
         - Reads `T3ChatConfig` from `AppState`.
         - Builds a DTO like:
           - `app_title`, `interface` flags.
           - Serialized `model_specs` list (without sensitive info).
           - Compact list of enabled providers with labels/icons (string keys only).
       - Ensure:
         - API keys and secrets **never** appear in the response.
     - Wire this endpoint into `server/src/main.rs` router and OpenAPI registration if applicable.

  2. **Models listing endpoint (Task B13)**
     - There is already `/api/v1/models` (see `server/src/api/v1/models/mod.rs`) that returns DB `AiModel`s.
     - Decide on approach:
       - **Option A**: Extend existing `/api/v1/models` to return a **config‑driven view** (e.g. `{ provider_key: ModelInfo[] }`) and deprecate the old shape.
       - **Option B**: Introduce a new endpoint, e.g. `GET /api/v1/config/models`, that wraps `ModelCatalog` while leaving existing endpoints intact.
     - Implement chosen option:
       - Response shape matching what the frontend expects (see UI dev section).
       - Backed by `ModelCatalog` to avoid per‑request calls to providers.

  3. **Backend testing**
     - Add/extend tests (or at least manual test scripts) to verify:
       - `t3chat.yaml` is correctly parsed and env interpolation works.
       - Provider registry returns reasonable `ModelInfo` lists for OpenAI and OpenRouter.
       - `/api/v1/config/startup` and models listing endpoints return the expected JSON.

---

## UI / Frontend Developer Plan

### 1. Types, Config Client, and Hooks – Tasks F1–F3

- **Goal**: Provide type‑safe access to startup config and models, and integrate selected `{provider, model}` (or `modelSpec`) into shared app state.

- **Files to add/update**
  - **Types**
    - `ui/src/types/config.ts` (new).
    - `ui/src/types/model.ts` (update; or split general model info vs config models if needed).
  - **API client**
    - `ui/src/lib/t3-chat-client.ts` (extend).
  - **Hooks**
    - `ui/src/hooks/useStartupConfig.ts` (new).
    - `ui/src/hooks/useModels.ts` (new or extend existing one, if already present).
  - **State store**
    - `ui/src/stores/appStore.ts` (extend for model selection state).

- **Implementation steps**
  1. **Define shared types (Task F1)**
     - In `ui/src/types/config.ts`, define:
       - `export interface InterfaceConfig { modelSelectEnabled: boolean; defaultModelSpec?: string; defaultProvider?: string; defaultModel?: string; }`
       - `export interface ProviderSummary { key: string; label: string; icon?: string; }`
       - `export interface ModelSpec { name: string; label: string; endpoint: string; model: string; parameters?: Record<string, unknown>; description?: string; icon?: string; default?: boolean; }`
       - `export interface StartupConfig { appTitle?: string; interface: InterfaceConfig; modelSpecs: ModelSpec[]; providers: ProviderSummary[]; }`
     - In `ui/src/types/model.ts`, add (or extend):
       - `export interface ProviderModels { [providerKey: string]: ModelInfo[]; }`
       - `export interface ModelInfo { id: string; displayName: string; provider: string; contextWindow?: number; supportsStreaming?: boolean; supportsImages?: boolean; }`
       - These should match the backend `/models` + `/config/startup` response shapes.

  2. **API client for config & models (Task F2)**
     - In `ui/src/lib/t3-chat-client.ts`, add:
       - `export async function getStartupConfig(): Promise<StartupConfig> { /* GET /api/v1/config/startup */ }`
       - `export async function getModels(): Promise<ProviderModels> { /* GET config‑backed models endpoint */ }`
     - Use the existing HTTP client pattern (whatever is used to call other T3Chat API endpoints).

  3. **React Query or custom hooks (Task F2/F3)**
     - If using React Query:
       - Create:
         - `useStartupConfig()` that calls `getStartupConfig()` and handles loading/error.
         - `useModels()` that calls `getModels()` and returns `{ data, isLoading, error }`.
     - If not using React Query, implement standard `useEffect`/`useState` hooks in:
       - `ui/src/hooks/useStartupConfig.ts`
       - `ui/src/hooks/useModels.ts`

  4. **Local state integration (Task F3)**
     - Extend `ui/src/stores/appStore.ts` (or similar global store) with:
       - `selectedProvider: string | null`
       - `selectedModel: string | null`
       - `selectedModelSpec: string | null`
       - Actions:
         - `setModelSelection({ provider, model, modelSpec })`
         - `resetModelSelectionToDefault()` (use defaults from `StartupConfig.interface`).
     - Ensure:
       - Chat pages and Models page both read from and write to this store so selection is consistent.

### 2. Model Selector UX – Tasks F4–F6

- **Goal**: Implement a `ModelSelector` component that uses startup config and model lists, usable globally (Models page) and per‑chat.

- **Files to add/update**
  - `ui/src/components/model/ModelSelector.tsx` (new or extend existing).
  - `ui/src/pages/Models.tsx` (enhance).
  - `ui/src/pages/Chat.tsx` / `ui/src/components/chat/*` (wire in selector per chat).

- **Implementation steps**
  1. **High‑level UX design (Task F4)**
     - Sketch how the selector is used in two contexts:
       - **Global default** on `Models` page.
       - **Per‑chat** selector in chat header/tool‑bar (e.g. next to conversation title).
     - Define flows:
       - **Spec‑first**: user picks from curated `modelSpecs` (default recommended path).
       - **Advanced**: user toggles to “Advanced” and selects provider → model.

  2. **Implement `ModelSelector` component (Task F5)**
     - In `ui/src/components/model/ModelSelector.tsx`:
       - Props:
         - `startupConfig`, `models`, `value`, `onChange`.
         - A `mode` prop to switch layout between compact (chat header) and full (Models page).
       - Behavior:
         - Shows a list of **model specs** in a primary section:
           - Each item displays label, provider icon, and description.
         - Below, shows **per‑provider** model lists (grouped by provider).
         - Supports search/filter by spec name, provider, model name.
         - Uses ShadCN UI components (e.g. `Popover`, `Command`, `Select`) consistent with existing T3Chat UI.

  3. **Integrate with Chat page (Task F4/F5)**
     - In `ui/src/pages/Chat.tsx` or header component:
       - Fetch `startupConfig` + `models` via hooks.
       - Read/write `selectedProvider/model/spec` via `appStore`.
       - Pass selection into existing chat send logic (so requests carry provider + model or spec).

  4. **Enhance Models page (Task F6)**
     - In `ui/src/pages/Models.tsx`:
       - Fetch `startupConfig` and `models`.
       - Render:
         - Overview of configured providers (from `providers` in `StartupConfig`).
         - Current default model/spec, with a `ModelSelector` instance for changing default.
         - Optionally badges showing which models are:
           - Config‑only (not yet discovered).
           - Discovered dynamically.
       - Wire “Set as default” to update user or app settings via existing backend APIs (or add new ones in a later iteration).

### 3. Provider‑Specific UI Enhancements – Tasks F7–F8

- **Goal**: Provide clear cues for OpenRouter/RouteLLM and good status/error feedback to users.

- **Files to add/update**
  - `ui/src/assets/provider-icons/*` (if you add SVGs).
  - `ui/src/components/model/ProviderBadge.tsx` (optional helper).
  - Existing toast/notification system (`ui/src/lib/toast.ts`) for errors.

- **Implementation steps**
  1. **Provider branding (Task F7)**
     - Add small icon assets or simple stylized initials for:
       - OpenRouter.
       - RouteLLM/Abacus.ai (name/logo to be finalized).
     - In `ModelSelector`, render a `ProviderBadge` next to model/spec labels using these icons.

  2. **RouteLLM “routes” UX (Task F7)**
     - When provider key is `routellm`:
       - Display entries as “routes” or “routing profiles” instead of generic models.
       - Add tooltips/secondary text describing what a route does (based on config metadata if available).

  3. **Error & status feedback (Task F8)**
     - In `useModels()` and `useStartupConfig()` hooks:
       - Capture errors when:
         - Provider is misconfigured (backend returns partial list).
         - Backend marks provider offline.
       - Expose these states to components.
     - In `ModelSelector` and Models page:
       - Show inline messages when a provider has no available models or is offline.
       - If user opens a conversation referencing a now‑disabled model:
         - Show a banner or toast explaining the situation.
         - Offer to switch to the default model/spec.

### 4. Frontend Testing & Integration

- **Goal**: Validate that the new config and model system works end‑to‑end.

- **Checks**
  - With a sample `t3chat.yaml`:
    - **Startup**:
      - App boots and `/api/v1/config/startup` returns expected JSON.
      - `/models` (or equivalent config models endpoint) returns per‑provider lists.
    - **UI**:
      - `ModelSelector` shows configured specs and models grouped by provider.
      - User can:
        - Change global default.
        - Change per‑chat selection.
      - Chat sends use the selected provider/model and work across OpenAI and OpenRouter.
    - **Error behavior**:
      - When a provider is disabled or API key missing, UI surfaces a clear message and offers fallback models.

---

## Coordination Notes (Backend ↔ UI)

- Before implementation, **agree on JSON contracts** for:
  - `GET /api/v1/config/startup` → `StartupConfig`.
  - Models endpoint (path + structure) → `ProviderModels`.
- Backend can **stub static responses** early so UI can start:
  - Implement `/api/v1/config/startup` with hard‑coded example values.
  - Implement models endpoint returning static example per‑provider lists.
- Once real `t3chat.yaml` loading and `ModelCatalog` are wired in, UI should require **no changes** beyond possible minor field additions.


