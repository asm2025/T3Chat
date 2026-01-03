# LibreChat vs T3Chat Feature Matrix & Parity Gap Analysis

**LibreChat Version**: v0.8.2-rc1
**Date**: Jan 3, 2026

## 1. Configuration (YAML)

| Feature | LibreChat Key | T3Chat Status | Notes |
| :--- | :--- | :--- | :--- |
| **Interface Flags** | `interface.*` | ✅ Mostly Present | `fileCitations`, `marketplace` are commented out in example. `agents` flag exists. |
| **Endpoints** | `endpoints.*` | ✅ Present | `custom`, `openAI`, `anthropic`, `google` supported. `assistants` config commented out. |
| **Model Specs** | `modelSpecs` | ✅ Present | Supported in YAML and UI. |
| **File Strategy** | `fileStrategy` | ⚠️ Partial | Structure exists in YAML but commented out. Need to verify implementation. |
| **MCP** | `mcpServers` | ✅ Present | Config structure matches. |
| **Web Search** | `webSearch` | ⚠️ Partial | Config keys present (`jinaApiKey` etc) but server implementation of search tool needs verification. |

## 2. API Endpoints (Client Contract vs Server)

Based on `client/src/lib/t3-chat-client.ts` expectations:

| Resource | Endpoint | Server Implementation | Action Required |
| :--- | :--- | :--- | :--- |
| **Auth/User** | `/v1/me`, `/v1/auth/*` | ✅ Present | `api/auth`, `api/user` |
| **Chats** | `/v1/chats` | ✅ Present | `api/chats` |
| **Messages** | `/v1/chats/:id/messages` | ✅ Present | `api/chats` |
| **Chat/Stream** | `/v1/chat/*` | ✅ Present | `api/chat` |
| **Models** | `/v1/models` | ✅ Present | `api/models` |
| **Config** | `/v1/config/*` | ✅ Present | `api/config` |
| **API Keys** | `/v1/keys` | ✅ Present | `api/user_api_keys` |
| **Files** | `/v1/files` | ⚠️ Partial | `api/files` exists. Need to check full CRUD + RAG hook. |
| **Presets** | `/v1/presets` | ❌ **MISSING** | Need `api/presets` module. |
| **Agents** | `/v1/agents` | ❌ **MISSING** | Need `api/agents` module. |
| **Assistants** | `/v1/assistants` | ❌ **MISSING** | Need `api/assistants` module. |
| **Tags** | `/v1/tags` | ❌ **MISSING** | Need `api/tags` module. |
| **Tools** | `/v1/tools` | ❌ **MISSING** | Need `api/tools` module. |
| **Search** | `/v1/search` | ❌ **MISSING** | Need MeiliSearch integration endpoint. |

## 3. Services & Infrastructure

| Service | LibreChat | T3Chat Target | Status |
| :--- | :--- | :--- | :--- |
| **Database** | MongoDB | Postgres (Normalized) | ✅ Done |
| **Vector DB** | pgvector | pgvector | ✅ Done (Docker) |
| **Search Engine**| MeiliSearch | MeiliSearch | ❌ Missing integration |
| **RAG API** | Python/FastAPI | Python/FastAPI | ⚠️ Container exists, connection missing |
| **File Storage** | Local/S3/Firebase | Local/S3/Firebase | ⚠️ Need to verify strategy implementation |

## 4. Immediate Action Plan (Phase 2)

1.  **Implement Missing Core Modules**:
    *   Presets
    *   Agents
    *   Tags
    *   Tools
    *   Assistants (stub/basic)
2.  **Integrate MeiliSearch**:
    *   Add `meilisearch-sdk` crate.
    *   Implement indexing hooks on Chat/Message CRUD.
    *   Add `/api/v1/search` endpoint.
3.  **Integrate RAG**:
    *   Connect `api/files` upload to `rag_api` ingestion.
    *   Add retrieval logic to Chat/Agent flows.

