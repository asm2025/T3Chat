# Build Instructions for T3Chat Desktop

## Prerequisites

1. **Flutter** with desktop support enabled
   ```bash
   flutter doctor
   flutter config --enable-windows-desktop  # or linux/macos
   ```

2. **Rust** and Cargo
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **flutter_rust_bridge_codegen**
   ```bash
   cargo install flutter_rust_bridge_codegen
   ```

## Build Steps

### 1. Install Flutter Dependencies

```bash
cd clients/desktop
flutter pub get
```

### 2. Generate Code (JSON serialization, Freezed, etc.)

```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

### 3. Build Rust Core

```bash
cd native/t3chat_core
cargo build --release
```

### 4. Generate Flutter Rust Bridge Bindings

```bash
cd clients/desktop
flutter_rust_bridge_codegen \
  --rust-input native/t3chat_core/src/ffi/bridge.rs \
  --dart-output lib/core/rust/bridge_generated.dart \
  --rust-output native/t3chat_core/src/bridge_generated.rs \
  --class-name RustLib
```

### 5. Run the App

```bash
flutter run -d windows  # or linux, macos
```

## Development Workflow

1. Make changes to Rust code in `native/t3chat_core/`
2. Rebuild Rust: `cd native/t3chat_core && cargo build`
3. Regenerate bridge: Run step 4 above
4. Hot reload Flutter app

## Troubleshooting

- If bridge generation fails, ensure `flutter_rust_bridge_codegen` is in your PATH
- If Rust build fails, check that all dependencies in `Cargo.toml` are available
- If FFI calls fail at runtime, ensure the Rust library is properly linked (check `CMakeLists.txt` or platform-specific build files)

