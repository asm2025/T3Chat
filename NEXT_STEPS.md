# Next Steps - Post Phase 2B

## Overview
Phase 2B (Frontend Multi-Provider Chat Interface) is complete. This document outlines the immediate next steps to get a working end-to-end chat experience.

## Priority 1: Backend Implementation (Phase 2A)

The backend needs to implement the following endpoints to support the frontend:

### 1. Chat Endpoints

#### POST `/api/v1/chat`
Non-streaming chat completion endpoint.

**Request:**
```typescript
{
  conversationId?: string,
  message: string,
  fileIds?: string[],
  endpointOptions: {
    endpoint: 'openai' | 'anthropic' | 'google' | 'custom',
    model: string,
    temperature?: number,
    maxTokens?: number,
    // ... other parameters
  }
}
```

**Response:**
```typescript
{
  messageId: string,
  text: string,
  finishReason?: string,
  tokenCount?: number,
  model: string
}
```

#### GET `/api/v1/chat/stream`
Server-Sent Events (SSE) streaming endpoint.

**Query Parameters:**
- Same as POST `/api/v1/chat` but in query string (URL-encoded JSON)

**SSE Events:**
```
event: chunk
data: {"type": "chunk", "delta": "text fragment"}

event: done
data: {"type": "done", "messageId": "uuid"}

event: error
data: {"type": "error", "error": "error message"}
```

### 2. File Endpoints

#### POST `/api/v1/files/upload`
Upload files for conversation.

**Request:** `multipart/form-data`
- `file`: File binary
- `conversationId`: UUID

**Response:**
```typescript
{
  id: string,
  filename: string,
  mimeType: string,
  sizeBytes: number,
  fileType: 'image' | 'document' | 'audio' | 'video' | 'other'
}
```

#### GET `/api/v1/files/:id`
Get file metadata.

#### DELETE `/api/v1/files/:id`
Delete file.

### 3. Conversation Endpoints

#### GET `/api/v1/conversations/:id/messages`
Load all messages for a conversation.

**Response:**
```typescript
{
  messages: Message[]
}
```

## Priority 2: Sidebar Integration

Update the app's Sidebar component to include ConversationList:

```tsx
// ui/src/components/sidebar.tsx
import { ConversationList } from '@/components/chat/ConversationList';

export function Sidebar() {
  return (
    <div>
      {/* Existing sidebar content */}
      
      {/* Add conversation list */}
      <ConversationList />
    </div>
  );
}
```

## Priority 3: Model Management

### Option A: Model Dropdown (Recommended)
Replace the text input in ChatView with a proper model selector:

1. Create `ModelDropdown` component
2. Fetch available models from `/api/v1/models?provider={endpoint}`
3. Show model metadata (context window, pricing, capabilities)
4. Filter models by provider

### Option B: Keep Text Input
If using text input, add:
- Model validation on backend
- Autocomplete suggestions
- Error feedback for invalid models

## Priority 4: Backend Provider Implementation

Implement the AI provider abstraction system (from plan.md Phase 2A):

### 1. Create AIProvider Trait
```rust
// server/src/ai/mod.rs
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest) 
        -> Result<Box<dyn Stream<Item = Result<StreamChunk>>>>;
}
```

### 2. Implement Providers
- `OpenAIProvider` - server/src/ai/providers/openai.rs
- `AnthropicProvider` - server/src/ai/providers/anthropic.rs
- `GoogleProvider` - server/src/ai/providers/google.rs

### 3. Create Provider Factory
```rust
// server/src/ai/factory.rs
pub fn create_provider(
    endpoint: &str,
    api_key: String,
) -> Result<Box<dyn AIProvider>> {
    match endpoint {
        "openai" => Ok(Box::new(OpenAIProvider::new(api_key))),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(api_key))),
        "google" => Ok(Box::new(GoogleProvider::new(api_key))),
        _ => Err(anyhow!("Unknown provider"))
    }
}
```

### 4. Implement Chat Route Handler
```rust
// server/src/api/v1/chat.rs
pub async fn chat_stream(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // 1. Get user's API key for provider
    // 2. Create provider instance
    // 3. Stream response
    // 4. Save messages to database
}
```

## Priority 5: Testing

### Frontend Testing
```bash
cd ui

# Install testing dependencies (if needed)
pnpm add -D @testing-library/react @testing-library/jest-dom vitest

# Run tests
pnpm test
```

**Test files to create:**
- `ChatView.test.tsx` - Component rendering and interactions
- `useLibreChatStreaming.test.ts` - Streaming hook logic
- `MessageBubble.test.tsx` - Multimodal content rendering
- `LibreChatMessageInput.test.tsx` - Input behavior

### Backend Testing
```bash
cd server

# Run tests
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

**Test files to create:**
- Provider tests (mock API calls)
- Repository tests (database operations)
- Integration tests (full flow)

### E2E Testing
```bash
# Install Playwright (if not already)
pnpm add -D @playwright/test

# Run E2E tests
pnpm playwright test
```

**Test scenarios:**
- Create new conversation
- Send message and receive response
- Switch between providers
- Upload files
- Stream message generation
- Cancel streaming

## Priority 6: Environment Configuration

### Backend Environment Variables
```bash
# server/.env.development
DATABASE_URL=postgresql://user:pass@localhost/t3chat
FIREBASE_PROJECT_ID=your-project-id
AUTO_MIGRATE=true

# API Keys (optional - users can provide their own)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=...
```

### Frontend Environment Variables
```bash
# ui/.env.development
VITE_API_URL=http://localhost:3000
```

## Quick Start Commands

### Start Development Environment
```bash
# Terminal 1: Start backend
cd server
cargo run

# Terminal 2: Start frontend
cd ui
pnpm dev
```

### Build for Production
```bash
# Backend
cd server
cargo build --release

# Frontend
cd ui
pnpm build
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend (React)                     │
├─────────────────────────────────────────────────────────────┤
│  ChatView                                                    │
│  ├── EndpointSelector (OpenAI, Anthropic, Google)          │
│  ├── EndpointSettings (Temperature, Tokens, etc.)          │
│  ├── MessageList                                            │
│  │   └── MessageBubble (Multimodal Content)                │
│  └── LibreChatMessageInput                                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ HTTP/SSE
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Backend (Rust/Axum)                     │
├─────────────────────────────────────────────────────────────┤
│  API Routes                                                  │
│  ├── POST /api/v1/chat          (non-streaming)            │
│  ├── GET  /api/v1/chat/stream   (SSE streaming)            │
│  ├── POST /api/v1/files/upload                             │
│  └── GET  /api/v1/conversations/:id/messages               │
├─────────────────────────────────────────────────────────────┤
│  AI Provider Abstraction                                    │
│  ├── OpenAIProvider                                         │
│  ├── AnthropicProvider                                      │
│  └── GoogleProvider                                         │
├─────────────────────────────────────────────────────────────┤
│  Repositories (Diesel)                                      │
│  ├── ConversationRepository                                 │
│  ├── MessageRepository                                      │
│  └── FileRepository                                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    PostgreSQL Database                       │
│  ├── conversations                                           │
│  ├── messages                                                │
│  ├── files                                                   │
│  ├── user_api_keys                                          │
│  └── ...                                                     │
└─────────────────────────────────────────────────────────────┘
```

## Success Criteria

### Minimal Working Demo
- [ ] User can create a new conversation
- [ ] User can select a provider (OpenAI)
- [ ] User can send a message
- [ ] User sees streaming response
- [ ] Message is saved to database
- [ ] User can load previous conversations

### Full Phase 2 Complete
- [ ] All 3 providers working (OpenAI, Anthropic, Google)
- [ ] File upload functional
- [ ] Settings panel persists to conversation
- [ ] Streaming with cancellation works
- [ ] Error handling graceful
- [ ] All tests passing

## Troubleshooting

### Frontend not connecting to backend
- Check VITE_API_URL in .env
- Ensure CORS is configured in backend
- Check browser console for errors

### Backend provider errors
- Verify API keys are set
- Check provider-specific requirements
- Review error logs in terminal

### Database connection issues
- Verify DATABASE_URL is correct
- Ensure PostgreSQL is running
- Check migrations have run

### Streaming not working
- Verify SSE endpoint is correct
- Check EventSource browser compatibility
- Ensure no CORS issues with SSE

## Resources

- [plan.md](plan.md) - Full development plan
- [PHASE_2B_IMPLEMENTATION_SUMMARY.md](PHASE_2B_IMPLEMENTATION_SUMMARY.md) - Frontend implementation details
- [SCHEMA_CHANGES_SUMMARY.md](SCHEMA_CHANGES_SUMMARY.md) - Database schema
- [README.md](README.md) - Project overview

---

**Last Updated:** December 1, 2025
**Next Phase:** Backend Implementation (Phase 2A)

