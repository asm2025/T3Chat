# T3Chat Desktop Implementation Summary

## Overview

The desktop client has been implemented according to the plan in `!ref/04. plan.clients.md`. It uses Flutter for the UI and a Rust core for API communication via `flutter_rust_bridge`.

## Architecture

### Rust Core (`native/t3chat_core/`)

- **API Client** (`src/api/client.rs`): HTTP client using `reqwest` with token management
- **Token Storage** (`src/storage/token_storage.rs`): Secure token storage using `keyring` crate
- **API Modules**:
  - `auth.rs`: Authentication (login, logout, get current user, get auth config)
  - `chat.rs`: Chat completion (send message, stream message)
  - `chats.rs`: Chat management (list, create, get, update, delete)
  - `user.rs`: User profile management
  - `models.rs`: Model listing
- **FFI Bindings** (`src/ffi/`): Bridge functions for Dart communication
- **Domain Models** (`src/domain/models.rs`): Data structures mirroring server API

### Flutter UI (`lib/`)

- **State Management**: Riverpod
- **Navigation**: go_router
- **Architecture**: Clean architecture with domain/data/presentation layers

#### Domain Layer
- **Models**: User, Chat, Message, AuthConfig, ChatMessageEvent
- **Repositories**: Abstract interfaces (AuthRepository, ChatRepository, UserRepository)

#### Data Layer
- **Repository Implementations**: Rust FFI-based implementations
- **Rust Client Wrapper**: Bridge to Rust core

#### Presentation Layer
- **Auth Feature**: Login screen with OIDC support
- **Chat List Feature**: Sidebar with chat list
- **Chat Detail Feature**: Message display and input with streaming support

## Key Features Implemented

1. ✅ Authentication (local username/password + OIDC)
2. ✅ Chat list with sidebar layout
3. ✅ Chat detail with message display
4. ✅ Message streaming (structure in place, needs bridge generation)
5. ✅ Token storage in Rust (secure, using keyring)
6. ✅ Error handling and state management

## Next Steps

### 1. Generate Flutter Rust Bridge Bindings

Run the code generation command (see `BUILD.md`):

```bash
flutter_rust_bridge_codegen \
  --rust-input native/t3chat_core/src/ffi/bridge.rs \
  --dart-output lib/core/rust/bridge_generated.dart \
  --rust-output native/t3chat_core/src/bridge_generated.rs \
  --class-name RustLib
```

### 2. Update Repository Implementations

After bridge generation, uncomment the FFI calls in:
- `lib/data/repositories/auth_repository_rust_impl.dart`
- `lib/data/repositories/chat_repository_rust_impl.dart`
- `lib/data/repositories/user_repository_rust_impl.dart`

### 3. Generate Dart Code

Run build_runner to generate JSON serialization and Freezed code:

```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

### 4. Platform-Specific Setup

For each platform (Windows/Linux/macOS), you'll need to:
- Link the Rust library in the Flutter build system
- Ensure the library is included in the app bundle

### 5. Testing

- Test authentication flow
- Test chat creation and listing
- Test message streaming
- Test error handling (network errors, auth errors)

## Known Limitations

1. **Streaming**: Currently collects all chunks before returning. For true streaming, consider using async FFI functions or a different approach.

2. **Runtime Management**: The FFI functions create a new Tokio runtime for each call. This works but is not optimal. Consider using a single shared runtime.

3. **Error Conversion**: Error conversion from Rust to Dart is basic. May need refinement based on actual error types.

4. **OIDC Callback**: The OIDC callback screen needs to properly extract and store the token from the URL.

## File Structure

```
clients/desktop/
├── lib/
│   ├── core/
│   │   ├── config/          # App configuration
│   │   ├── error/            # Error types
│   │   ├── providers/        # Repository providers
│   │   ├── routing/          # App router
│   │   └── rust/             # Rust bridge (generated)
│   ├── domain/
│   │   ├── models/           # Domain models
│   │   └── repositories/     # Repository interfaces
│   ├── data/
│   │   └── repositories/     # Repository implementations
│   ├── features/
│   │   ├── auth/             # Authentication feature
│   │   └── chat/             # Chat features
│   └── main.dart
├── native/
│   └── t3chat_core/          # Rust core crate
│       └── src/
│           ├── api/          # API client and modules
│           ├── domain/       # Domain models and errors
│           ├── ffi/          # FFI bindings
│           └── storage/      # Token storage
└── pubspec.yaml
```

## Dependencies

### Flutter
- `flutter_riverpod`: State management
- `go_router`: Navigation
- `flutter_rust_bridge`: FFI bridge
- `freezed`, `json_annotation`: Code generation
- `url_launcher`: OIDC login

### Rust
- `reqwest`: HTTP client
- `tokio`: Async runtime
- `serde`, `serde_json`: Serialization
- `keyring`: Secure token storage
- `flutter_rust_bridge`: FFI bridge
- `thiserror`: Error handling

## Configuration

API base URL is configured in `lib/core/config/app_config.dart`. Default is `http://localhost:3000` for development.

