## T3Chat Model & Provider Configuration Plan

This document outlines how to bring LibreChat‑style, YAML‑driven model and provider configuration into **T3Chat**, split into parallelizable **backend (Rust / server)** and **frontend (React / UI)** workstreams.

We will:
- Introduce `t3chat.yaml` and `t3chat.example.yaml` for configuration.
- Support updated versions of LibreChat’s built‑in providers/models (OpenAI, Anthropic, Google, Bedrock, Azure if applicable).
- Add first‑class support for **OpenRouter** and, where feasible, **Abacus.ai RouteLLM** as configurable providers.
- Respect T3Chat’s **relational DB** and **Rust backend** architecture.

---

## 1. High‑Level Design & Deliverables

- **Config‑driven model system**
  - Single `t3chat.yaml` file read on server startup (and optionally reloadable).
  - Defines:
    - Global interface flags (e.g. whether model selection UI is enabled).
    - Provider/endpoint definitions (OpenAI, Anthropic, Google, Bedrock, Azure, OpenRouter, RouteLLM, custom HTTP providers).
    - Per‑provider model lists: statically configured or fetched dynamically via provider APIs.
    - Optional **model specs** (curated presets that bundle endpoint + model + parameters, similar to LibreChat `modelSpecs`).
  - `t3chat.example.yaml` documents the schema and provides realistic examples.

- **Backend API surface**
  - `GET /api/v1/config/startup` (or similar) returns:
    - Interface flags.
    - Model specs.
    - Enabled endpoints/providers and high‑level capabilities.
  - `GET /api/v1/models` returns:
    - Per‑endpoint model lists (merged from provider discovery + YAML overrides).
  - Existing chat/preset/agent APIs read from DB but **respect current config** (e.g. reject models not allowed by config).

- **Frontend model selection**
  - A `ModelSelector` component (or enhancement of existing Models page & chat input) that:
    - Uses startup config for model specs & interface flags.
    - Uses `/models` to list models per provider.
    - Allows picking either:
      - A **spec** (preconfigured preset), or
      - A **provider + raw model ID** pair.

- **DB & migration alignment**
  - Ensure relational tables (`ai_provider`, `ai_model`, `preset`, `agent`, etc.) are consistent with:
    - Config‑enabled providers/models.
    - Versioning/metadata (capabilities, families, pricing hints if needed).
  - Migrations to add missing columns/relations if we need additional metadata surfaced from YAML.

---

## 2. Backend (Rust / Server) Workstream

### 2.1 Configuration Schema & Loader (`t3chat.yaml`)

- **Task B1: Define Rust config structs & schema**
  - Design a `Config` Rust type that mirrors relevante LibreChat concepts but is idiomatic for T3Chat:
    - `interface` section (flags like `model_select_enabled`, etc.).
    - `endpoints` or `providers` section, with entries like:
      - `openai`, `anthropic`, `google`, `bedrock`, `azure`, `openrouter`, `routellm`, and an optional `custom` list for generic HTTP providers.
    - Each provider config should support:
      - `api_key` reference (string that may include `${ENV_VAR}` syntax).
      - `base_url` (optional for built‑ins, required for custom, OpenRouter, RouteLLM).
      - `models` sub‑object:
        - `default: [model_id, ...]`
        - `fetch: bool` – whether to discover models at runtime via the provider’s API.
        - `user_id_query: bool` or equivalent if needed (like LibreChat).
      - Optional extras (title model, drop/add params, etc.) that map cleanly to Rust types.
    - Optional `model_specs` section:
      - List of entries `{ name, label, endpoint, model, parameters, description?, icon?, default? }`.
  - Decide how strictly we validate YAML (e.g. `serde` + custom validation).

- **Task B2: YAML loading & environment interpolation**
  - Implement a loader that:
    - Reads `t3chat.yaml` (configurable path, e.g. via env var `T3CHAT_CONFIG`).
    - Optionally falls back to defaults when file missing.
    - Performs `${VAR}` interpolation using environment variables for secrets and URLs.
  - Add error handling:
    - Fatal errors (e.g. malformed YAML, missing required keys) vs. warnings (e.g. unknown provider name).

- **Task B3: Example configuration file**
  - Create `t3chat.example.yaml` containing:
    - Documented sections for:
      - Interface options.
      - Built‑in providers (OpenAI, Anthropic, Google, Bedrock, Azure).
      - OpenRouter example.
      - RouteLLM / Abacus.ai example (see section 2.4).
      - Custom provider example.
      - `model_specs` examples showing how to create curated presets.
    - Inline comments explaining each field and referencing T3Chat docs.

### 2.2 Provider & Model Discovery Service

- **Task B4: Provider registry abstraction**
  - Introduce a provider registry layer (Rust) that:
    - Maps config provider keys to concrete provider clients (existing `ai::providers::openai`, `anthropic`, `google`, etc.).
    - Normalizes naming (e.g. canonical enums or string constants for `OPENAI`, `ANTHROPIC`, etc.).
    - Exposes trait(s) like:
      - `list_models() -> Result<Vec<ModelInfo>, Error>`
      - `supports_assistants()`, `supports_images()`, etc. (even if initially stubbed).
  - Integrate existing providers (OpenAI, Anthropic, Google) into this trait.

- **Task B5: Dynamic model discovery**
  - For providers that support model listing (OpenAI, OpenRouter, maybe others):
    - Implement `list_models` that calls the provider’s API (or uses known model lists if the provider does not expose a listing endpoint).
  - For config entries with `models.fetch: true`:
    - Call `list_models` at startup for that provider.
    - Fallback to `models.default` when API errors or returns empty.
  - For `models.fetch: false`:
    - Use `models.default` as the canonical list.

- **Task B6: Merge default/provider‑discovered models with YAML overrides**
  - Implement a `ModelCatalog` service responsible for:
    - Loading **default models** from providers.
    - Overriding/augmenting them with config‑provided lists per provider.
    - Returning `HashMap<ProviderKey, Vec<ModelInfo>>`.
  - Define conflict resolution policy:
    - For example, YAML lists override provider defaults, or union with explicit exclusions.
  - Expose this catalog as an application‑wide singleton or cached service.

### 2.3 DB Integration & Migrations

- **Task B7: Align `ai_provider` and `ai_model` tables with config**
  - Review existing schemas in `server/src/db/models` and `server/migrations`:
    - Determine which fields map to config:
      - Provider ID/name, type, base URL, capabilities.
      - Model ID, display name, provider FK, capabilities (text‑only, vision, tools).
  - Decide:
    - Whether providers/models are fully config‑driven (ephemeral, not stored) or mirrored in DB for analytics/permissions.
  - If mirrored:
    - Add migration(s) to:
      - Add any missing fields (e.g. `family`, `is_deprecated`, `is_default`, `source`).
      - Possibly add a `config_source` field to track YAML vs system vs dynamic.

- **Task B8: Sync config to DB (optional but recommended)**
  - Implement a sync routine that:
    - On startup, reconciles `t3chat.yaml` and provider discovery results with DB tables.
    - Adds new providers/models if missing.
    - Optionally marks missing ones as disabled/deprecated rather than deleting.
  - Ensure idempotency and log all changes for observability.

- **Task B9: Enforce config in chat flows**
  - In existing chat/agent APIs (under `server/src/api/v1`):
    - Validate requested `{provider, model}` against the `ModelCatalog` (and optionally DB).
    - Return clear errors when a model is not configured or has been disabled.
  - Wire this validation into any preset/agent creation/update endpoints as well.

### 2.4 New Providers: OpenRouter & RouteLLM / Abacus.ai

- **Task B10: OpenRouter provider support**
  - Backend:
    - Add an `OpenRouter` provider implementation under `ai::providers`:
      - Handles base URL `https://openrouter.ai/api/v1`.
      - Auth header(s) per OpenRouter spec.
      - `list_models()` using OpenRouter’s `/models` endpoint (if available).
    - Integrate into provider registry and `ModelCatalog`.
  - Config:
    - Define `openrouter` section in `t3chat.yaml` with:
      - `api_key: "${OPENROUTER_API_KEY}"`
      - `base_url: "https://openrouter.ai/api/v1"`
      - `models: { default: [...], fetch: true }`.
    - Add worked example in `t3chat.example.yaml`.

- **Task B11: RouteLLM / Abacus.ai support**
  - Research phase (no code yet):
    - Confirm current RouteLLM / Abacus.ai routing API:
      - Does it expose a list of routed “models” or “routes”?
      - How to specify which underlying models to use?
    - Decide whether we treat it as:
      - A **single logical provider** (e.g. `routellm` with 1–N route IDs), or
      - A simple “meta‑endpoint” over one of the existing providers.
  - Design phase:
    - Specify config shape in `t3chat.yaml`, e.g.:
      - `routellm: { api_key, base_url, routes: { default: [...], fetch: bool } }`.
    - Decide how `list_models()` behaves:
      - Returns routes as “models” (e.g. `route/general`, `route/coding`).
  - Once design is locked:
    - Add provider implementation and integrate into registry & `ModelCatalog`.
    - Add examples to `t3chat.example.yaml`.

### 2.5 Startup Config & Public API Endpoints

- **Task B12: Startup config endpoint**
  - Implement `GET /api/v1/config/startup` that returns:
    - App title/branding (if any).
    - Interface flags from `t3chat.yaml` (e.g. whether model selection is visible).
    - `model_specs` list (serialized from config).
    - Basic list of enabled providers/endpoints (names and icons/labels).
  - Ensure response includes only non‑sensitive configuration (no API keys).

- **Task B13: Models listing endpoint**
  - Implement `GET /api/v1/models` requiring authenticated user:
    - Returns a map `{ provider_key: [model_id, ...] }`.
    - Optionally enriched with simple metadata (e.g. friendly name, category).
  - Backed by `ModelCatalog` to avoid per‑request provider calls.

---

## 3. Frontend (React / UI) Workstream

### 3.1 Types, Config Client, and Hooks

- **Task F1: Shared types for config & models**
  - Define TS interfaces mirroring the backend `startup` and `models` responses:
    - `StartupConfig`, `EndpointConfig`, `ModelSpec`, `ModelsResponse`, etc.
  - Ensure they align with `t3chat.yaml` schema and Rust types.

- **Task F2: API client for config & models**
  - Extend `ui/src/lib/t3-chat-client.ts` (or equivalent) to include:
    - `getStartupConfig(): Promise<StartupConfig>`
    - `getModels(): Promise<ModelsResponse>`
  - Add React Query hooks, e.g.:
    - `useStartupConfig()`
    - `useModels()`

- **Task F3: Local state integration**
  - Decide where to store:
    - Selected `{provider, model}` for current conversation.
    - Selected model spec (if any).
  - Integrate with existing stores (`stores/appStore.ts`, chat‑related stores) so chat pages and settings share the same selection model.

### 3.2 Model Selector UI (Global + Per‑Chat)

- **Task F4: High‑level UX design**
  - Design how users pick models:
    - Global default on the **Models** page.
    - Per‑conversation selector on the **Chat** page header or input area.
  - Handle two flows:
    - Selecting a **model spec** (preset): simple label, icon, recommended parameters.
    - Selecting a **provider + raw model**: two‑step selection (endpoint, then model id).

- **Task F5: Model selector component**
  - Implement a `ModelSelector` similar to LibreChat’s:
    - Uses `useStartupConfig` and `useModels`.
    - Renders:
      - A section for `modelSpecs` with labels/icons.
      - A section for each provider with its model list.
    - Supports search/filter over specs, providers, and model names.
    - Shows provider icons (OpenAI, Anthropic, Google, Bedrock, Azure, OpenRouter, RouteLLM, etc.).
  - Integrate with:
    - Chat creation and conversation switching.
    - Preset/agent creation UI, if present.

- **Task F6: Models page UX**
  - Enhance `ui/src/pages/Models.tsx` to:
    - Display list of providers and models from `/models`.
    - Allow setting a **default model spec** or `{provider, model}` combination.
    - Optionally, show which models are configured via `t3chat.yaml` vs discovered dynamically.

### 3.3 Provider‑Specific UI Enhancements

- **Task F7: OpenRouter & RouteLLM UI**
  - Add recognizable branding/icons for:
    - OpenRouter.
    - RouteLLM/Abacus.ai (once logo & naming are finalized).
  - If RouteLLM uses “routes” instead of models:
    - Display them as “routing profiles” within the selector, with tooltips explaining behavior.

- **Task F8: Error & status feedback**
  - Surface when:
    - A provider is configured but offline (API errors in `/models`).
    - A requested model is no longer available (config changed since conversation created).
  - Provide clear inline messages and fallbacks (e.g. revert to default model).

---

## 4. Parallelization Strategy

To support multiple developers working in parallel:

- **Backend Track A (Config & Loader)**
  - Owns Tasks **B1–B3, B12–B13**.
  - Deliverables:
    - `t3chat.yaml` schema & loader.
    - `t3chat.example.yaml`.
    - `GET /config/startup` and `GET /models` returning stubbed/static data initially.

- **Backend Track B (Providers & Model Catalog)**
  - Owns Tasks **B4–B6, B10–B11**.
  - Can work off the types/interfaces defined by Track A.
  - Delivers provider trait, provider registry, dynamic discovery, and OpenRouter/RouteLLM integrations.

- **Backend Track C (DB & Validation Integration)**
  - Owns Tasks **B7–B9**.
  - Coordinates DB migrations and updates chat APIs to enforce config/model availability.

- **Frontend Track D (API & Hooks)**
  - Owns Tasks **F1–F3**.
  - Can start as soon as `/config/startup` and `/models` response shapes are sketched (even before backend is fully implemented, using mocks).

- **Frontend Track E (Model Selector & Pages)**
  - Owns Tasks **F4–F8**.
  - Builds `ModelSelector`, integrates into Chat and Models pages, and handles provider‑specific UX.

Each track can be implemented and tested independently, with well‑defined JSON contracts between backend and frontend derived from the `t3chat.yaml` schema.

---

## 5. Next Steps for Refinement

- Finalize `t3chat.yaml` schema draft (field names, nesting, provider keys).
- Decide whether provider/model data is persisted in DB or purely config/runtime.
- Confirm RouteLLM / Abacus.ai capabilities and finalize its config and API shape.
- Once the above are agreed, we can:
  - Lock in the JSON contract for `/config/startup` and `/models`.
  - Start implementing backend & frontend tracks according to this plan.


