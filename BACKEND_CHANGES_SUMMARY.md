# Backend API Changes Summary - Frontend Migration Guide

## Overview
The backend has been refactored to remove legacy "Conversation" models and consolidate everything to use "Chat" models directly. All compatibility/mapping code has been removed. This document outlines the changes that may affect the frontend.

## API Response Structure Changes

### 1. Chat List Response (`GET /api/v1/chats`)
**CHANGED**: Now returns paginated results with a different structure:

**Before** (if existed):
```json
{
  "chats": [...]
}
```

**After**:
```json
{
  "data": [
    {
      "id": "uuid",
      "userId": "string",
      "title": "string",
      "modelProvider": "string",  // Derived from endpoint field
      "modelId": "string",         // From model field
      "createdAt": "ISO8601",
      "updatedAt": "ISO8601"
    }
  ],
  "total": 0
}
```

**Query Parameters**:
- `page` (optional): Page number
- `pageSize` (optional): Items per page

**Important**: The `modelProvider` field is now derived by parsing the `endpoint` string from the database. The backend converts endpoint values like "openai", "anthropic", etc. to the provider string.

### 2. Chat Response (`GET /api/v1/chats/{id}`)
**Structure unchanged**, but note:
- `title` is now guaranteed to be a string (defaults to "New Chat" if null)
- `modelProvider` is parsed from the `endpoint` field
- `modelId` comes from the `model` field

### 3. Message Response Structure
**IMPORTANT CHANGES**:

```typescript
interface MessageResponse {
  id: string;
  chatId: string;
  role: string;              // "user", "assistant", "system", "tool"
  content: string;            // From text field in DB
  metadata?: any;            // From content (JSONB) field in DB
  parentMessageId?: string;
  sequenceNumber: number;     // ⚠️ Always 0 - not stored in new model
  createdAt: string;
  tokensUsed?: number;        // From token_count
  modelUsed?: string;         // From model field
}
```

**Key Changes**:
- `sequenceNumber` is **always 0** in responses (not calculated/stored in new schema)
- `content` comes from the `text` field in the database
- `metadata` comes from the `content` (JSONB) field in the database
- Field mapping: `text` → `content`, `content` (JSONB) → `metadata`

## API Request Changes

### 1. Create Chat (`POST /api/v1/chats`)
**Request body unchanged**:
```json
{
  "title": "string (optional)",
  "modelProvider": "string",
  "modelId": "string"
}
```

### 2. Update Chat (`PUT /api/v1/chats/{id}`)
**Request body unchanged**:
```json
{
  "title": "string (optional)"
}
```

### 3. Create Message (`POST /api/v1/chats/{id}/messages`)
**Request body unchanged**:
```json
{
  "content": "string",
  "role": "string (optional)"  // "user", "assistant", "system"
}
```

### 4. Update Message (`PUT /api/v1/chats/{chatId}/messages/{id}`)
**Request body unchanged**:
```json
{
  "content": "string (optional)",
  "metadata": "object (optional)"
}
```

## Error Handling Changes

**CHANGED**: Error responses now use `anyhow::Error` instead of structured error types.

**Before** (if existed):
- Specific error types like `DbError::NotFound`

**After**:
- All errors return generic HTTP status codes
- Error messages are checked for "not found" string to determine 404 vs 500
- No structured error response format

**Error Status Codes**:
- `404`: When error message contains "not found"
- `500`: All other errors

## Endpoints Summary

All endpoints remain the same:

- `GET /api/v1/chats` - List chats (now paginated)
- `POST /api/v1/chats` - Create chat
- `GET /api/v1/chats/{id}` - Get chat with messages
- `PUT /api/v1/chats/{id}` - Update chat
- `DELETE /api/v1/chats/{id}` - Delete chat (soft delete via archiving)
- `GET /api/v1/chats/{id}/messages` - List messages
- `POST /api/v1/chats/{id}/messages` - Create message
- `PUT /api/v1/chats/{chatId}/messages/{id}` - Update message
- `DELETE /api/v1/chats/{chatId}/messages/{id}` - Delete message
- `DELETE /api/v1/chats/{id}/messages` - Clear all messages

## Frontend Action Items

### 1. Update Chat List Handling
- ⚠️ **CRITICAL**: Backend returns `{ data: T[], total: number }` but client expects `PaginatedResponse<T>` with `{ data: T[], total: number, page: number, pageSize: number }`
  - **Fix**: Either update backend to include `page` and `pageSize` in response, OR update client `PaginatedResponse` type to make `page` and `pageSize` optional
- ✅ Add support for `page` and `pageSize` query parameters (already supported in client)
- ⚠️ **IMPORTANT**: Backend `ChatResponse` uses `modelProvider` and `modelId` fields, but client `Chat` type uses `endpoint` and `model` fields
  - **Fix**: Update client types or add mapping layer to convert between API response and internal Chat type

### 2. Update Message Handling
- ⚠️ **IMPORTANT**: `sequenceNumber` is now always `0` - remove any logic that depends on this field
- ⚠️ **IMPORTANT**: Backend `MessageResponse` uses `content` (from DB `text` field) and `metadata` (from DB `content` JSONB field)
  - Client `Message` type uses `text` field - may need mapping
  - Verify message display logic uses correct field
- ✅ Ensure message display uses `content` field from API response

### 3. Type Definitions
Update TypeScript interfaces to match:

```typescript
interface ChatListResponse {
  data: ChatResponse[];
  total: number;
}

interface ChatResponse {
  id: string;
  userId: string;
  title: string;
  modelProvider: string;
  modelId: string;
  createdAt: string;
  updatedAt: string;
}

interface MessageResponse {
  id: string;
  chatId: string;
  role: string;
  content: string;
  metadata?: any;
  parentMessageId?: string;
  sequenceNumber: number;  // Always 0
  createdAt: string;
  tokensUsed?: number;
  modelUsed?: string;
}
```

### 4. Error Handling
- ✅ Update error handling to work with generic HTTP status codes
- ✅ Remove any code that expects structured error types
- ✅ Handle 404 vs 500 based on status code only

### 5. Pagination
- ✅ Add pagination support to chat list if not already present
- ✅ Use `page` and `pageSize` query parameters
- ✅ Display total count from `total` field

## Testing Checklist

- [ ] Chat list displays correctly with pagination
- [ ] Chat creation works with modelProvider/modelId
- [ ] Messages display correctly (content from text field)
- [ ] Message metadata is accessible from metadata field
- [ ] sequenceNumber is not used in any business logic
- [ ] Error handling works for 404 and 500 cases
- [ ] Pagination controls work correctly
- [ ] Chat update/delete operations work
- [ ] Message create/update/delete operations work

## Breaking Changes Summary

1. **Chat List Response**: Changed from array to paginated object with `data` and `total` (missing `page` and `pageSize` in response)
2. **Chat Response Fields**: Backend returns `modelProvider`/`modelId`, but client expects `endpoint`/`model` - type mismatch
3. **Message sequenceNumber**: Always 0, should not be used for ordering
4. **Error Format**: No structured error types, only HTTP status codes
5. **Field Mapping**: Message `content` comes from DB `text`, `metadata` comes from DB `content` (JSONB)
6. **Message Response**: Backend returns `content` field, but client `Message` type may use `text` field

## Specific Client Code Issues Found

### File: `client/src/lib/t3-chat-client.ts`
- Line 193: `chats.list()` expects `PaginatedResponse<ChatWithTags>` but backend returns `{ data: ChatResponse[], total: number }` (missing `page` and `pageSize`)

### File: `client/src/types/librechat.ts`
- `Chat` interface uses `endpoint: Endpoint` and `model: string`
- Backend `ChatResponse` uses `modelProvider: string` and `modelId: string`
- Need to add mapping or update types

### File: `client/src/stores/appStore.ts`
- Line 618: `t3ChatClient.chats.get(chatId)` - verify response structure matches `ChatWithTags`
- Line 619: `t3ChatClient.messages.list(chatId)` - verify response structure matches `LibreChatMessage[]`

## Non-Breaking Changes

- All endpoint paths remain the same
- Request body structures remain the same
- Response field names remain the same (camelCase)
- Authentication/authorization unchanged

