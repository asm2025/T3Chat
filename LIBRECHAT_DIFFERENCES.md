# T3Chat vs LibreChat - Differences Tracking

This document tracks the key differences between T3Chat and LibreChat to help maintain feature parity and understand architectural decisions.

**Last Updated**: 2025-01-03  
**LibreChat Version Tracked**: v0.8.2-rc1  
**T3Chat Version**: Current development

---

## 🏗️ **Architectural Differences**

### Backend Technology Stack

| Component | LibreChat | T3Chat | Notes |
|-----------|-----------|--------|-------|
| **Backend Language** | Node.js (Express/Hono) | Rust (Axum) | Performance and type safety benefits |
| **Database** | MongoDB | PostgreSQL (normalized) | Fully normalized relational schema |
| **ORM** | Mongoose/MongoDB Driver | Diesel (async) | Type-safe SQL queries |
| **Search Engine** | MeiliSearch | MeiliSearch | ✅ Same |
| **Vector DB** | pgvector | pgvector | ✅ Same |
| **RAG API** | Python/FastAPI | Python/FastAPI (external) | ✅ Same (external service) |

### Database Schema Differences

| Concept | LibreChat | T3Chat | Notes |
|---------|-----------|--------|-------|
| **Primary Entity** | `Conversation` | `Chat` | Terminology difference - T3Chat uses "Chat" throughout |
| **Message Storage** | Embedded in Conversation | Separate `messages` table | Normalized approach |
| **User Data** | Embedded documents | Normalized tables with FKs | Better referential integrity |
| **API Keys** | Encrypted in user doc | Separate `user_api_keys` table | Better isolation |
| **Presets** | Embedded in user | Separate `presets` table | Supports sharing and reordering |
| **Tags** | Embedded arrays | Separate `tags` + `chat_tags_map` | Many-to-many relationship |
| **Agents** | Embedded documents | Separate `agents` + junction tables | Better tool/action management |

### API Route Differences

| Aspect | LibreChat | T3Chat | Notes |
|--------|-----------|--------|-------|
| **API Versioning** | `/api/v1/...` | `/api/...` | T3Chat uses unversioned routes |
| **Route Structure** | RESTful with versioning | RESTful without versioning | Simpler URL structure |
| **Terminology** | `conversation`, `conversationId` | `chat`, `chatId` | T3Chat uses "Chat" terminology |

---

## 🔄 **Feature Parity Status**

### ✅ **Implemented Features**

| Feature | LibreChat | T3Chat | Status |
|---------|-----------|--------|--------|
| **Multi-Provider Chat** | ✅ | ✅ | OpenAI, Anthropic, Google, OpenRouter, Custom |
| **Streaming Chat** | ✅ | ✅ | Server-Sent Events (SSE) |
| **Presets** | ✅ | ✅ | CRUD, reorder, set default |
| **Agents** | ✅ | ✅ | CRUD, tools, chat starters |
| **Tags** | ✅ | ✅ | CRUD, reorder, attach to chats |
| **Tools** | ✅ | ✅ | List, get, system/user tools |
| **Assistants** | ✅ | ✅ | List, get, sync (stub) |
| **File Upload** | ✅ | ✅ | Upload, list, delete, STT |
| **MeiliSearch** | ✅ | ✅ | Full-text search across chats/messages |
| **RAG Integration** | ✅ | ✅ | File ingestion and retrieval |
| **OIDC Auth** | ✅ | ✅ | JWKS-based JWT verification |
| **Local Auth** | ✅ | ✅ | Username/password with bcrypt |
| **API Key Management** | ✅ | ✅ | Encrypted storage per user/provider |

### ⚠️ **Partially Implemented Features**

| Feature | LibreChat | T3Chat | Status | Notes |
|---------|-----------|--------|--------|-------|
| **Web Search Tool** | ✅ | ⚠️ | Partial | Config exists, tool implementation needs verification |
| **File Citations** | ✅ | ⚠️ | Partial | RAG integration exists, UI citations need implementation |
| **MCP Servers** | ✅ | ⚠️ | Partial | Config structure matches, runtime support pending |
| **Image Generation** | ✅ | ❌ | Missing | DALL-E, Stable Diffusion, Flux support |
| **Code Interpreter** | ✅ | ❌ | Missing | Sandboxed code execution |
| **Import/Export** | ✅ | ❌ | Missing | Chat import/export functionality |
| **Shared Links** | ✅ | ⚠️ | Partial | Database schema exists, API endpoints pending |
| **TTS (Text-to-Speech)** | ✅ | ❌ | Missing | STT exists, TTS missing |
| **Balance/Transactions** | ✅ | ⚠️ | Partial | Schema exists, API endpoints pending |

### ❌ **Not Yet Implemented Features**

| Feature | LibreChat | T3Chat | Priority | Notes |
|---------|-----------|--------|----------|-------|
| **Marketplace** | ✅ | ❌ | Low | Agent/preset marketplace |
| **People Picker** | ✅ | ❌ | Low | User selection for sharing |
| **Multi-Conversation View** | ✅ | ❌ | Medium | Side-by-side chat view |
| **Reasoning UI** | ✅ | ❌ | Medium | Thinking/reasoning visualization |
| **Generative UI** | ✅ | ❌ | Low | Code artifacts, Mermaid diagrams |
| **Speech & Audio** | ✅ | ⚠️ | Medium | STT ✅, TTS ❌ |
| **Advanced File Handling** | ✅ | ⚠️ | Medium | Basic upload ✅, advanced features pending |

---

## 🔧 **Configuration Differences**

### YAML Configuration

| Aspect | LibreChat | T3Chat | Notes |
|--------|-----------|--------|-------|
| **Config File** | `librechat.yaml` | `t3chat.yaml` | Compatible schema |
| **Terminology** | `titleConvo` | `titleChat` | Uses "Chat" terminology |
| **Placeholders** | `${ENV_VAR}` | `${ENV_VAR}` | ✅ Same syntax |
| **Interface Flags** | `interface.*` | `interface.*` | ✅ Same structure |

### Environment Variables

| Variable | LibreChat | T3Chat | Notes |
|----------|-----------|--------|-------|
| **Database** | `MONGO_URI` | `DATABASE_URL` | PostgreSQL connection string |
| **Search** | `MEILI_HOST`, `MEILI_MASTER_KEY` | `MEILI_HOST`, `MEILI_MASTER_KEY` | ✅ Same |
| **RAG** | `RAG_API_URL` | `RAG_API_URL` | ✅ Same |
| **Auth** | Various OIDC vars | Same OIDC vars | ✅ Same |

---

## 📊 **Data Model Mapping**

### Core Entities

| LibreChat | T3Chat | Mapping Notes |
|-----------|--------|---------------|
| `Conversation` | `Chat` | Direct mapping, different terminology |
| `Message` | `Message` | ✅ Same concept |
| `User` | `User` | ✅ Same concept |
| `Preset` | `Preset` | ✅ Same concept |
| `Agent` | `Agent` | ✅ Same concept |
| `Assistant` | `Assistant` | ✅ Same concept |
| `Tag` | `Tag` | ✅ Same concept |
| `Tool` | `Tool` | ✅ Same concept |
| `Action` | `Action` | ✅ Same concept |

### Relationship Differences

| Relationship | LibreChat | T3Chat | Notes |
|--------------|-----------|--------|-------|
| **Chat ↔ Messages** | Embedded array | Foreign key | T3Chat uses normalized approach |
| **Chat ↔ Tags** | Embedded array | Junction table | T3Chat supports many-to-many |
| **Agent ↔ Tools** | Embedded array | Junction table | T3Chat supports per-agent tool config |
| **User ↔ API Keys** | Embedded array | Separate table | T3Chat better isolation |

---

## 🚀 **Performance & Scalability**

| Aspect | LibreChat | T3Chat | Notes |
|--------|-----------|--------|-------|
| **Concurrency** | Node.js event loop | Rust async/await | T3Chat better for CPU-bound tasks |
| **Memory Usage** | Higher (V8) | Lower (native) | T3Chat more efficient |
| **Type Safety** | TypeScript (compile-time) | Rust (compile + runtime) | T3Chat stronger guarantees |
| **Database Queries** | MongoDB queries | SQL with indexes | T3Chat better query optimization |

---

## 🔐 **Security Differences**

| Aspect | LibreChat | T3Chat | Notes |
|--------|-----------|--------|-------|
| **API Key Encryption** | ✅ | ✅ | Both use encryption |
| **JWT Verification** | ✅ | ✅ | Both support OIDC/JWKS |
| **SQL Injection** | N/A (MongoDB) | Protected (Diesel) | T3Chat type-safe queries |
| **Input Validation** | Runtime checks | Compile-time + runtime | T3Chat stronger validation |

---

## 📝 **API Compatibility**

### Request/Response Format

| Aspect | LibreChat | T3Chat | Compatibility |
|--------|-----------|--------|---------------|
| **Request Format** | JSON | JSON | ✅ Compatible |
| **Response Format** | JSON | JSON | ✅ Compatible |
| **Streaming** | SSE | SSE | ✅ Compatible |
| **Error Format** | JSON error objects | JSON error objects | ✅ Compatible |

### Endpoint Compatibility

T3Chat accepts `conversationId` as an alias for `chatId` in request bodies for compatibility, but all internal code and responses use `chatId`.

---

## 🐛 **Known Limitations**

### T3Chat-Specific

1. **No MongoDB Migration Tool**: T3Chat cannot directly import from LibreChat's MongoDB database. Users must export/import via JSON.
2. **No Neon Database Support**: T3Chat uses standard PostgreSQL protocol, not Neon's serverless HTTP driver.
3. **No Cloudflare Workers**: T3Chat backend is standalone only (no serverless deployment).

### LibreChat Features Not Yet Ported

1. **Code Interpreter**: Sandboxed code execution environment
2. **Image Generation**: DALL-E, Stable Diffusion, Flux integration
3. **Advanced MCP**: Full MCP server runtime support
4. **Marketplace**: Agent/preset sharing marketplace
5. **Import/Export**: Chat import/export functionality

---

## 📚 **Migration Guide**

### From LibreChat to T3Chat

1. **Export Data**: Use LibreChat's export feature to export conversations as JSON
2. **Import to T3Chat**: Use T3Chat's import endpoint (when implemented) or manual import
3. **Update API Keys**: Re-enter API keys in T3Chat (encrypted storage)
4. **Recreate Presets**: Manually recreate presets or use import (when implemented)
5. **Update OIDC Config**: Update OIDC redirect URIs to point to T3Chat

### Terminology Mapping

When migrating or comparing:
- LibreChat `Conversation` = T3Chat `Chat`
- LibreChat `conversationId` = T3Chat `chatId`
- LibreChat `/api/v1/...` = T3Chat `/api/...`

---

## 🔄 **Update Process**

### How to Keep T3Chat Updated with LibreChat Changes

1. **Monitor LibreChat Releases**:
   - Check [LibreChat Releases](https://github.com/danny-avila/LibreChat/releases)
   - Review `CHANGELOG.md` in LibreChat repo
   - Note new features, bug fixes, and breaking changes

2. **Update This Document**:
   - Add new features to "Not Yet Implemented" section
   - Move features to "Implemented" as they're added
   - Update version numbers and dates

3. **Review Configuration Changes**:
   - Compare `librechat.example.yaml` with `t3chat.example.yaml`
   - Add new config keys to T3Chat's config schema
   - Update environment variable documentation

4. **Review API Changes**:
   - Check LibreChat's API documentation
   - Compare with T3Chat's API routes
   - Add missing endpoints to implementation plan

5. **Review Database Schema Changes**:
   - Check LibreChat migrations (if accessible)
   - Compare with T3Chat's normalized schema
   - Plan migrations for new features

6. **Test Compatibility**:
   - Test import/export (when implemented)
   - Verify API compatibility
   - Check configuration compatibility

---

## 📋 **Feature Implementation Checklist**

Use this checklist when implementing new LibreChat features:

- [ ] Review LibreChat implementation
- [ ] Map to T3Chat architecture (Chat vs Conversation)
- [ ] Design normalized database schema (if needed)
- [ ] Implement backend API endpoints
- [ ] Add frontend UI components
- [ ] Update configuration schema (`t3chat.yaml`)
- [ ] Add environment variables documentation
- [ ] Update this differences document
- [ ] Test feature parity
- [ ] Update API documentation

---

## 🔗 **Resources**

- **LibreChat**: [GitHub](https://github.com/danny-avila/LibreChat) | [Documentation](https://docs.librechat.ai/)
- **T3Chat**: Current repository
- **MeiliSearch**: [Documentation](https://www.meilisearch.com/docs)
- **RAG API**: LibreChat's RAG service (see `rag.yml` in LibreChat repo)
- **pgvector**: [Documentation](https://github.com/pgvector/pgvector)

---

**Note**: This document should be updated whenever:
- New LibreChat features are released
- T3Chat implements new features
- Architecture decisions are made
- Breaking changes occur in either project

