# Database Schema Normalization - Changes Summary

## Overview

The database schema has been **fully normalized** for PostgreSQL following relational database best practices. **No lookup/reference data in JSONB or arrays** - everything uses proper tables, foreign keys, and indexes.

## Key Principle: Relational Data in Relational Tables

**Commandment**: **Thou shalt not use JSONB or arrays for lookup/reference data!**

**What we normalized:**
- ✅ Tags → `tags` table + `conversation_tags_map` junction table
- ✅ Tools → `tools` table + `agent_tools`/`assistant_tools` junction tables
- ✅ Agent relationships → `agent_hierarchy`, `project_agents`, `agent_actions` junction tables
- ✅ Conversation starters → `agent_conversation_starters`, `assistant_conversation_starters` tables

**What stays as JSONB (and why):**
- `model_parameters` - Truly dynamic, varies wildly by provider (OpenAI ≠ Anthropic ≠ Google)
- `feature_flags` - Optional boolean flags, provider-specific
- `tool_resources` - Provider-specific IDs (e.g., OpenAI vector store IDs)
- `assistants.tools` - OpenAI API compatibility (their specific format)

**What stays as UUID[] (and why):**
- `file_ids` - Ordered lists where order matters semantically
- Always loaded together, no metadata needed per relationship

## Performance Philosophy: Proper Indexes > Denormalization

**Hottest query**: "Load conversation with messages" (happens on every conversation view)
```sql
-- This query should be FAST (no unnecessary JOINs)
SELECT * FROM conversations WHERE id = ? AND user_id = ?;
SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at;
-- ✅ No changes here, still fast!
```

**Lookup queries** (also fast with proper indexes):
```sql
-- "Find all conversations with tag 'work'"
SELECT c.* FROM conversations c
JOIN conversation_tags_map ctm ON c.id = ctm.conversation_id  
JOIN tags t ON ctm.tag_id = t.id
WHERE t.user_id = ? AND t.name = 'work';
-- ✅ Index seek on both tag_name and conversation_id - very fast!
```

---

## Major Changes

### 1. Added New Tables

#### `ai_models` (Reference Table)
**Why**: Centralize model metadata without forcing FK constraints
```sql
- provider, model_id, display_name
- context_window, max_output_tokens
- supports_streaming, supports_images, supports_functions
- cost_per_input_token, cost_per_output_token
```

**Usage Pattern**:
- Messages store `model` as TEXT (not FK) for historical accuracy
- `ai_models` table is for UI/configuration/pricing reference
- If model is removed from table, historical messages unaffected

#### `user_api_keys` (Security)
**Why**: Encrypted storage of user API keys per provider
```sql
- user_id, provider, encrypted_key (AES-256-GCM)
- is_default, last_used_at
```

---

### 2. Removed Redundant Fields

#### messages.user_id ✅ REMOVED
**Before**: Stored in both `messages` and `conversations`
**After**: Get from `conversations` table
```sql
-- Old way:
SELECT * FROM messages WHERE user_id = ?;

-- New way (if needed):
SELECT m.* FROM messages m
JOIN conversations c ON m.conversation_id = c.id
WHERE c.user_id = ?;
```
**Savings**: 16 bytes per message (UUID), improved data integrity

#### messages.summary_token_count ✅ REMOVED
**Why**: Can calculate if needed, rarely used

#### messages.unfinished ✅ REMOVED  
**Why**: Derivable from `finish_reason IS NULL`

#### messages.icon_url ✅ REMOVED
**Why**: Get from agent/assistant/model table

---

### 3. Consolidated Fields into JSONB

#### conversations.model_parameters (was 10+ separate columns)
**Before**:
```sql
- temperature: DOUBLE PRECISION
- top_p: DOUBLE PRECISION
- top_k: INTEGER
- max_tokens: INTEGER
- max_output_tokens: INTEGER
- presence_penalty: DOUBLE PRECISION
- frequency_penalty: DOUBLE PRECISION
- stop_sequences: TEXT[]
- reasoning_effort: TEXT
```

**After**:
```sql
- model_parameters: JSONB
```

**Example value**:
```json
{
  "temperature": 0.7,
  "top_p": 1.0,
  "top_k": 40,
  "max_tokens": 2048,
  "presence_penalty": 0,
  "frequency_penalty": 0,
  "stop_sequences": []
}
```

**Benefits**:
- Cleaner schema (1 column instead of 10+)
- Provider-agnostic (different providers have different params)
- Easy to extend without migrations
- Still queryable with JSONB operators

#### conversations.feature_flags (was 6+ boolean columns)
**Before**:
```sql
- resend_files: BOOLEAN
- resend_images: BOOLEAN
- image_detail: TEXT
- prompt_cache: BOOLEAN
- thinking: BOOLEAN
- thinking_budget: INTEGER
```

**After**:
```sql
- feature_flags: JSONB
```

**Benefits**: Same as model_parameters - cleaner, extensible, provider-agnostic

---

### 3. Replaced Arrays with Junction Tables (Proper Normalization)

#### Tags: From Array to Proper Many-to-Many
**Before**:
```sql
conversations: tags TEXT[]  -- Array of tag names directly in table
```

**After**:
```sql
-- Lookup table for tag definitions
tags:
  - id: UUID PRIMARY KEY
  - user_id: TEXT FK
  - name: TEXT NOT NULL
  - color: TEXT
  - description: TEXT
  - position: INTEGER
  - UNIQUE(user_id, name)

-- Junction table for conversation-tag relationships  
conversation_tags_map:
  - conversation_id: UUID FK (conversations)
  - tag_id: UUID FK (tags)
  - PRIMARY KEY (conversation_id, tag_id)
```

**Benefits**:
- ✅ Query "all conversations with tag X" efficiently with proper indexes
- ✅ Tag metadata (color, description) stored once, not duplicated
- ✅ Rename tag → updates all conversations automatically
- ✅ Referential integrity (can't reference non-existent tag)
- ✅ Can track tag usage count, creation date, etc.

**Query examples**:
```sql
-- Before (array): Slow, can't use indexes well
SELECT * FROM conversations WHERE 'work' = ANY(tags);

-- After (junction table): Fast with proper indexes
SELECT c.* FROM conversations c
JOIN conversation_tags_map ctm ON c.id = ctm.conversation_id
JOIN tags t ON ctm.tag_id = t.id
WHERE t.user_id = ? AND t.name = 'work';
```

#### Tools: System & User Tools with Metadata
**Before**:
```sql
agents: tools TEXT[]  -- Array of tool names like ['web_search', 'code_interpreter']
assistants: tools JSONB  -- OpenAI format (keep for API compat)
```

**After**:
```sql
-- Tool definitions (system + user-created)
tools:
  - id: UUID PRIMARY KEY
  - name: TEXT UNIQUE NOT NULL
  - display_name: TEXT NOT NULL
  - description: TEXT
  - tool_type: TEXT CHECK (IN 'system', 'plugin', 'function', 'action')
  - is_system: BOOLEAN
  - configuration_schema: JSONB

-- Agent-tool relationships with per-agent configuration
agent_tools:
  - agent_id: UUID FK (agents)
  - tool_id: UUID FK (tools)
  - configuration: JSONB
  - is_enabled: BOOLEAN
  - PRIMARY KEY (agent_id, tool_id)

-- Assistant-tool relationships
assistant_tools:
  - assistant_id: UUID FK (assistants)
  - tool_id: UUID FK (tools)
  - configuration: JSONB
  - PRIMARY KEY (assistant_id, tool_id)
```

**Benefits**:
- ✅ Centralized tool catalog
- ✅ Per-agent tool configuration
- ✅ Can enable/disable tools without modifying agent
- ✅ Can query "which agents use this tool"
- ✅ Can add new tools without modifying existing agents
- ✅ Tool metadata (description, icon, schema) in one place

**Note**: `assistants.tools` stays JSONB for OpenAI API compatibility

#### Multi-Agent Systems: Hierarchical Relationships
**Before**:
```sql
agents:
  - sub_agent_ids: UUID[]  -- Array of child agent IDs
  - project_ids: UUID[]    -- Array of project IDs
  - action_ids: UUID[]     -- Array of action IDs
```

**After**:
```sql
-- Agent parent-child hierarchy
agent_hierarchy:
  - parent_agent_id: UUID FK (agents)
  - sub_agent_id: UUID FK (agents)
  - order_index: INTEGER
  - PRIMARY KEY (parent_agent_id, sub_agent_id)
  - CHECK (parent_agent_id != sub_agent_id)  -- Prevent self-reference

-- Project-agent membership
project_agents:
  - project_id: UUID FK (projects)
  - agent_id: UUID FK (agents)
  - role: TEXT  -- e.g., "reviewer", "coder", "coordinator"
  - order_index: INTEGER
  - PRIMARY KEY (project_id, agent_id)

-- Agent-action associations
agent_actions:
  - agent_id: UUID FK (agents)
  - action_id: UUID FK (actions)
  - is_enabled: BOOLEAN
  - PRIMARY KEY (agent_id, action_id)
```

**Benefits**:
- ✅ Can add metadata (role, order, enabled status)
- ✅ Bidirectional queries: "agents in project" AND "projects using agent"
- ✅ Prevent circular references (CHECK constraint)
- ✅ Can track when relationships were created
- ✅ Better performance for complex multi-agent queries

#### Conversation Starters: Manageable Lists
**Before**:
```sql
agents: conversation_starters TEXT[]
assistants: conversation_starters TEXT[]
```

**After**:
```sql
agent_conversation_starters:
  - id: UUID PRIMARY KEY
  - agent_id: UUID FK (agents)
  - text: TEXT NOT NULL
  - order_index: INTEGER
  - created_at: TIMESTAMPTZ

assistant_conversation_starters:
  - id: UUID PRIMARY KEY
  - assistant_id: UUID FK (assistants)
  - text: TEXT NOT NULL
  - order_index: INTEGER
  - created_at: TIMESTAMPTZ
```

**Benefits**:
- ✅ Update individual starters without rewriting entire array
- ✅ Can add metadata later (usage count, effectiveness metrics)
- ✅ Cleaner INSERT/UPDATE/DELETE operations
- ✅ Can track when each starter was added

---

### 4. UUID Arrays: ONLY for Ordered Lists (Not Lookup Data)

**✅ When UUID[] is appropriate**:
```sql
messages: file_ids UUID[]     -- Order matters (first file = primary context)
assistants: file_ids UUID[]   -- Order matters for OpenAI API
```

**Why it's OK here**:
- Order is semantically meaningful
- Files are always loaded together with parent entity
- FK constraint validates all UUIDs exist
- Better performance than JOIN for this specific use case
- No need for metadata per file-message relationship

**❌ When to use junction tables instead**:
- Tags → need metadata (color, description), need to query both directions
- Tools → need per-agent configuration, need to manage tool catalog
- Agent relationships → need metadata (role, order), need bidirectional queries

---

### 5. Changed File References to UUID Arrays

#### messages.file_ids (was `files` JSONB)
**Before**:
```sql
- files: JSONB  -- Array of {id, filename, url, ...}
```

**After**:
```sql
- file_ids: UUID[]  -- References files.id
```

**Benefits**:
- Referential integrity (can't reference non-existent file)
- Easier queries: `WHERE ? = ANY(file_ids)`
- GIN index for fast lookups
- No data duplication
- Preserves file order (important for context)

---

### 6. Simplified Role/Sender Fields

#### messages.role (replaces `sender` and consolidates with `is_created_by_user`)
**Before**:
```sql
- sender: TEXT  -- "user", "assistant", "gpt-4", etc.
- is_created_by_user: BOOLEAN
```

**After**:
```sql
- role: TEXT CHECK (role IN ('user', 'assistant', 'system', 'tool'))
- is_created_by_user: BOOLEAN  -- Still needed for UI logic
```

**Benefits**:
- Consistent enum-like values
- Database-enforced valid values
- Simpler logic

---

### 7. Denormalized Fields (Kept for Performance)

#### ❌ DID NOT REMOVE: messages.model and messages.endpoint

**Why these look redundant but aren't**:

1. **Users can switch models mid-conversation**
   ```
   Conversation starts with: gpt-4-turbo
   Message 1: user asks question
   Message 2: assistant replies (model: gpt-4-turbo)
   [User switches to claude-3-opus]
   Message 3: user asks question
   Message 4: assistant replies (model: claude-3-opus)
   ```

2. **Historical accuracy**
   - Must preserve exact model/endpoint used for each message
   - Even if model is deprecated/removed later

3. **Query performance**
   - Most common query: "Load conversation with all messages"
   - Adding JOIN to `ai_models` would slow this down
   - Model name is small (10-30 bytes)

4. **Flexibility**
   - Support custom OpenAI-compatible endpoints
   - Support provider-specific model naming

**Decision**: Keep as TEXT, NOT foreign key to `ai_models`

---

## Performance Optimizations

### 1. Strategic Indexes

#### Composite indexes for common queries
```sql
-- Conversation list (most common):
CREATE INDEX idx_conversations_user_updated 
  ON conversations(user_id, updated_at DESC);

-- Archived conversations:
CREATE INDEX idx_conversations_user_archived 
  ON conversations(user_id, is_archived, updated_at DESC);

-- Messages by conversation (critical path):
CREATE INDEX idx_messages_conversation_id 
  ON messages(conversation_id, created_at);
```

#### Partial indexes for filtered queries
```sql
-- Only index active models:
CREATE INDEX idx_ai_models_active 
  ON ai_models(is_active) WHERE is_active = true;

-- Only index temporary files:
CREATE INDEX idx_files_temporary 
  ON files(expires_at) WHERE is_temporary = true;
```

#### GIN indexes for arrays and full-text search
```sql
-- Array containment queries (only for ordered lists):
CREATE INDEX idx_messages_file_ids ON messages USING GIN(file_ids);

-- Full-text search:
CREATE INDEX idx_conversations_title_fts 
  ON conversations USING GIN(to_tsvector('english', title));
CREATE INDEX idx_messages_text_fts 
  ON messages USING GIN(to_tsvector('english', COALESCE(text, '')));
CREATE INDEX idx_tools_name_fts 
  ON tools USING GIN(to_tsvector('english', display_name));
CREATE INDEX idx_agents_name_fts 
  ON agents USING GIN(to_tsvector('english', name));

-- Junction table indexes (B-tree, very fast):
CREATE INDEX idx_conversation_tags_map_tag_id ON conversation_tags_map(tag_id);
CREATE INDEX idx_agent_tools_tool_id ON agent_tools(tool_id);
CREATE INDEX idx_project_agents_agent_id ON project_agents(agent_id);
```

### 2. Proper CASCADE Rules

```sql
-- User deletes → cascade to all user data:
conversations.user_id → ON DELETE CASCADE
messages.conversation_id → ON DELETE CASCADE

-- Optional references → set null:
conversations.agent_id → ON DELETE SET NULL
messages.parent_message_id → ON DELETE SET NULL
```

---

## Storage Savings Estimate

For a typical conversation with 50 messages:

**Before (per message)**:
- user_id (UUID): 16 bytes
- summary_token_count: 4 bytes
- unfinished: 1 byte
- icon_url (avg): 50 bytes
- files JSONB (duplication): ~200 bytes (if has files)
**Total redundant**: ~271 bytes/message

**After**: All removed or normalized
**Savings**: 271 bytes × 50 messages = **13.55 KB per conversation**

For 100,000 conversations: **1.36 GB saved**

Plus better query performance from:
- Fewer columns to scan
- Better index utilization
- Reduced JOIN overhead (user_id removal)

---

## Migration Impact

### Breaking Changes
1. `user_id` removed from messages (queries must JOIN)
2. Multiple AI param fields → single JSONB
3. File references changed from JSONB to UUID[]

### Application Code Updates Needed
```rust
// Before:
let user_id = message.user_id;

// After:
let user_id = conversation.user_id;  // Get from conversation
```

```rust
// Before:
let temp = conversation.temperature;
let max_tokens = conversation.max_tokens;

// After:
let params: ModelParameters = serde_json::from_value(conversation.model_parameters)?;
let temp = params.temperature;
let max_tokens = params.max_tokens;
```

---

## Summary Table: All Changes

| Table | Field | Change | Reason |
|-------|-------|--------|--------|
| **NEW** | `ai_models` | Added table | Model metadata/reference |
| **NEW** | `user_api_keys` | Added table | Encrypted API key storage |
| **NEW** | `tags` | Added table | User-defined tags (was `conversation_tags`) |
| **NEW** | `conversation_tags_map` | Added junction table | Many-to-many: conversations ↔ tags |
| **NEW** | `tools` | Added table | System & user tools catalog |
| **NEW** | `agent_tools` | Added junction table | Many-to-many: agents ↔ tools |
| **NEW** | `assistant_tools` | Added junction table | Many-to-many: assistants ↔ tools |
| **NEW** | `agent_actions` | Added junction table | Many-to-many: agents ↔ actions |
| **NEW** | `project_agents` | Added junction table | Many-to-many: projects ↔ agents |
| **NEW** | `agent_hierarchy` | Added junction table | Many-to-many: agents ↔ sub-agents |
| **NEW** | `agent_conversation_starters` | Added table | Ordered list of agent starters |
| **NEW** | `assistant_conversation_starters` | Added table | Ordered list of assistant starters |
| users | Multiple fields | Consolidated to JSONB | Cleaner schema |
| conversations | AI params (10+ fields) | → `model_parameters JSONB` | Provider-agnostic, extensible |
| conversations | Feature flags (6+ fields) | → `feature_flags JSONB` | Cleaner, extensible |
| conversations | tags TEXT[] | ✅ REMOVED → junction table | Proper normalization |
| conversations | agent_id/assistant_id | TEXT → UUID FK | Proper referential integrity |
| messages | user_id | ✅ REMOVED | Get from conversation JOIN |
| messages | sender | → `role` with CHECK constraint | Consistent values |
| messages | summary_token_count | ✅ REMOVED | Rarely used |
| messages | unfinished | ✅ REMOVED | Derivable from finish_reason |
| messages | icon_url | ✅ REMOVED | Get from agent/model |
| messages | files JSONB | → `file_ids UUID[]` | Ordered list + FK validation |
| messages | parent_message_id | TEXT → UUID FK | Proper type |
| messages | model/endpoint | ❌ KEPT as TEXT | Performance + history |
| presets | All AI params | → `model_parameters JSONB` | Match conversations |
| agents | tools TEXT[] | ✅ REMOVED → junction table | Proper normalization |
| agents | action_ids UUID[] | ✅ REMOVED → junction table | Proper normalization |
| agents | sub_agent_ids UUID[] | ✅ REMOVED → junction table | Proper normalization |
| agents | project_ids UUID[] | ✅ REMOVED → junction table | Proper normalization |
| agents | conversation_starters TEXT[] | ✅ REMOVED → separate table | Manageable list |
| assistants | conversation_starters TEXT[] | ✅ REMOVED → separate table | Manageable list |
| assistants | tools JSONB | ❌ KEPT as JSONB | OpenAI API compatibility |
| assistants | file_ids UUID[] | ❌ KEPT as UUID[] | Ordered list |
| tool_calls | Multiple fields | Normalized structure | Clearer purpose |
| transactions | Fields | Added input/output tokens | Better billing tracking |

---

## Questions Answered

### Q: Why not make `model` a foreign key to `ai_models`?
**A**: Historical accuracy + performance. Messages must preserve the exact model string used, even if that model is later deprecated. Also avoids JOIN on the hottest query path.

### Q: Why use JSONB for model_parameters instead of a separate table?
**A**: 
1. Different providers have different parameters (not uniform schema)
2. Parameters are always loaded together (no benefit to normalizing)
3. JSONB is fast and queryable in PostgreSQL
4. Easier to extend without migrations

### Q: Why keep `is_created_by_user` if we have `role`?
**A**: UI logic often needs quick boolean check. `role` provides precise type, `is_created_by_user` provides convenient boolean.

### Q: What about query performance without `user_id` in messages?
**A**: 
- Most queries are: "Get messages for THIS conversation" (no user_id needed)
- Rare query "Get all user messages" can JOIN (acceptable performance)
- Benefit: Data integrity guaranteed (can't have message with wrong user_id)

### Q: Why use junction tables instead of UUID[] arrays for tags/tools/agents?
**A**:
**Arrays are fine for**:
- Ordered lists where order matters (e.g., file_ids - first file = primary context)
- Data always loaded together with parent
- No metadata needed per relationship

**Junction tables are required for**:
- Lookup/reference data (tags, tools) - need to query "all conversations with tag X"
- Metadata per relationship (tool configuration per agent, role in project)
- Bidirectional queries (agents in project AND projects using agent)
- Data integrity (can rename tag, update all references automatically)

### Q: Won't junction tables make queries slower?
**A**: 
- **No!** With proper indexes, junction tables are FASTER for lookup queries
- Example: "Find conversations with tag 'work'" 
  - Array: Full table scan, can't use index effectively
  - Junction table: Index seek on tag name, then index seek on conversation IDs
- PostgreSQL is optimized for JOINs, not array operations
- Indexes on junction table PRIMARY KEYs make lookups very fast

---

## Next Steps

1. ✅ Schema designed and documented
2. ⏭️ Create Diesel migration with all tables
3. ⏭️ Generate Rust models with proper types
4. ⏭️ Update repositories to use new schema
5. ⏭️ Update API handlers to serialize/deserialize JSONB fields

---

**Document Version**: 1.0  
**Date**: November 30, 2025  
**Status**: Schema design complete, ready for implementation

