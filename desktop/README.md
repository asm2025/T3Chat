# T3Chat Desktop Client

Flutter desktop client for T3Chat with Rust core for API communication.

## Setup

1. Install Flutter with desktop support
2. Install Rust and Cargo
3. Install `flutter_rust_bridge_codegen`:
   ```bash
   cargo install flutter_rust_bridge_codegen
   ```

4. Install dependencies:
   ```bash
   flutter pub get
   cd native/t3chat_core && cargo build
   ```

5. Generate FFI bindings:
   ```bash
   flutter pub run build_runner build
   ```

6. Run the app:
   ```bash
   flutter run -d windows  # or linux, macos
   ```

## Architecture

- **Rust Core** (`native/t3chat_core/`): Handles all HTTP communication, token storage, and API calls
- **Flutter UI**: Provides the user interface using Riverpod for state management and go_router for navigation
- **FFI Bridge**: `flutter_rust_bridge` connects Rust and Dart

