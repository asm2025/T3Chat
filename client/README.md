# T3Chat Client

Unified Flutter client for T3Chat supporting both desktop (Windows, Linux, macOS) and mobile (Android, iOS) platforms.

## Overview

The T3Chat client uses a platform-aware architecture that automatically selects the appropriate data layer implementation:

-   **Desktop platforms**: Uses Rust FFI via `flutter_rust_bridge` for API communication
-   **Mobile platforms**: Uses Dio HTTP client with interceptors for API communication

This unified codebase allows you to develop and maintain a single Flutter application that works across all platforms.

## Architecture

### Platform Detection

The app automatically detects the platform at runtime and selects the appropriate implementation:

```dart
// Desktop: Rust FFI implementation
if (kIsWeb || Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
  return AuthRepositoryRustImpl();
}
// Mobile: Dio HTTP implementation
else {
  return AuthRepositoryImpl(apiClient);
}
```

### Data Layer

-   **Desktop**: Rust core (`native/t3chat_core/`) with FFI bindings
-   **Mobile**: Dio HTTP client with interceptors (auth, error handling, logging)

### Domain Layer

Shared across all platforms:

-   **Models**: User, Chat, Message, Model, AuthConfig, ChatMessageEvent
-   **Repositories**: Abstract interfaces for Auth, Chat, User, Models

### Presentation Layer

Shared UI code that works on all platforms:

-   Authentication screens and widgets
-   Chat list and detail screens
-   Shared widgets (loading, error, empty state)

## Prerequisites

### For All Platforms

1. **Flutter SDK** (3.0.0 or higher)

    ```bash
    flutter doctor
    ```

2. **Dart SDK** (included with Flutter)

### For Desktop Development

1. **Rust and Cargo**

    ```bash
    rustc --version
    cargo --version
    ```

2. **flutter_rust_bridge_codegen**

    ```bash
    cargo install flutter_rust_bridge_codegen
    ```

3. **Platform-specific tools**:
    - **Windows**: Visual Studio with C++ build tools
    - **Linux**: GCC, pkg-config, libgtk-3-dev
    - **macOS**: Xcode Command Line Tools

### For Mobile Development

1. **Android Studio** (for Android)

    - Android SDK
    - Android emulator or physical device

2. **Xcode** (for iOS, macOS only)
    - iOS Simulator or physical device
    - CocoaPods

## Setup

### 1. Platform-Specific Setup

#### Desktop Setup

**Important**: For desktop development, you must build the Rust core and generate the bridge bindings **before** running `flutter pub get`, as the generated bridge files are referenced by your Dart code.

1. **Build Rust Core**

    ```bash
    cd native/t3chat_core
    cargo build --release
    cd ../..
    ```

2. **Generate Flutter Rust Bridge Bindings**

    **Recommended**: Run from the client root directory:

    ```bash
    # From client/ directory
    flutter_rust_bridge_codegen generate --config-file native/t3chat_core/flutter_rust_bridge.yaml
    ```

    **Note**: This step must be completed before running `flutter pub get` because the generated bridge files are imported by your Dart code. Running from the client root avoids path resolution issues on Windows.

#### Mobile Setup

1. **Android**: No additional setup required (Gradle will handle dependencies)

2. **iOS** (macOS only):
    ```bash
    cd ios
    pod install
    cd ..
    ```

### 2. Install Flutter Dependencies

```bash
cd client
flutter pub get
```

**Note**: For desktop development, ensure you've completed step 1 (Rust bridge generation) before running this command.

If dependency resolution fails:

```bash
# Windows
rmdir /s /q .dart_tool
del /q pubspec.lock
flutter pub get

# Linux/macOS
rm -rf .dart_tool
rm pubspec.lock
flutter pub get
```

### 3. Generate Code

Generate JSON serialization and other generated code:

```bash
dart run build_runner build --delete-conflicting-outputs
```

**Note**: This step should be run after `flutter pub get` has successfully installed all dependencies.

### 4. Configure API Base URL

Update `lib/core/config/app_config.dart`:

```dart
enum Environment { dev, staging, prod }

class AppConfig {
  static const Environment env = Environment.dev;
  // ...
}
```

Default API URLs:

-   **Dev**: `http://localhost:3000`
-   **Staging**: `https://staging-api.example.com`
-   **Prod**: `https://api.example.com`

**Note**: For Android emulator, use `10.0.2.2` instead of `localhost` to access the host machine.

## Running the App

### Desktop

```bash
# Windows
flutter run -d windows

# Linux
flutter run -d linux

# macOS
flutter run -d macos
```

### Mobile

```bash
# Run on connected device/emulator
flutter run

# Run on specific device
flutter run -d <device-id>

# List available devices
flutter devices
```

## Project Structure

```
client/
├── lib/
│   ├── core/
│   │   ├── config/          # App configuration (merged)
│   │   ├── error/           # Error types (merged)
│   │   ├── logging/         # Logging utilities (from mobile)
│   │   ├── network/         # Dio API client (mobile only)
│   │   │   └── interceptors/ # Auth, error, logging interceptors
│   │   ├── providers/       # Platform-aware repository providers
│   │   ├── routing/         # Router with auth redirects (from mobile)
│   │   └── rust/            # Rust bridge (desktop only)
│   ├── domain/
│   │   ├── models/          # Merged domain models
│   │   └── repositories/    # Repository interfaces (merged)
│   ├── data/
│   │   └── repositories/    # Both Rust and Dio implementations
│   │       ├── *_rust_impl.dart  # Desktop implementations
│   │       └── *_impl.dart       # Mobile implementations
│   ├── features/            # Merged features
│   │   ├── auth/
│   │   └── chat/
│   ├── shared/              # Shared widgets (from mobile)
│   │   └── widgets/
│   └── main.dart            # Merged entry point
├── native/
│   └── t3chat_core/         # Rust core (desktop only)
├── android/                 # Android platform (from mobile)
├── ios/                     # iOS platform (if exists)
├── windows/                  # Windows platform (desktop)
├── linux/                    # Linux platform (desktop)
├── macos/                    # macOS platform (desktop)
└── pubspec.yaml             # Merged dependencies
```

## Dependencies

### Core Dependencies

-   `flutter_riverpod`: State management
-   `go_router`: Navigation
-   `dio`: HTTP client (mobile)
-   `flutter_rust_bridge`: FFI bridge (desktop)
-   `flutter_secure_storage`: Secure token storage (mobile)
-   `json_annotation`: JSON serialization
-   `freezed_annotation`: Immutable data classes
-   `logger`: Logging utilities
-   `url_launcher`: OIDC login

### Dev Dependencies

-   `build_runner`: Code generation
-   `json_serializable`: JSON serialization code generation
-   `freezed`: Immutable data class generation
-   `riverpod_generator`: Riverpod code generation (optional)
-   `flutter_lints`: Linting rules

## Features

-   ✅ **Platform-aware architecture**: Automatic selection of Rust FFI (desktop) or Dio HTTP (mobile)
-   ✅ **Authentication**: Local username/password and OIDC support
-   ✅ **Chat management**: List, create, update, delete chats
-   ✅ **Message streaming**: Real-time message streaming with SSE
-   ✅ **Secure storage**: Keyring (desktop) or flutter_secure_storage (mobile)
-   ✅ **Error handling**: Comprehensive error handling with domain exceptions
-   ✅ **Logging**: Configurable logging with debug mode support

## Development Workflow

### Desktop Development

1. Make changes to Rust code in `native/t3chat_core/`
2. Rebuild Rust: `cd native/t3chat_core && cargo build`
3. Regenerate bridge: Run flutter_rust_bridge_codegen
4. Hot reload Flutter app

### Mobile Development

1. Make changes to Dart code
2. Regenerate code if models changed: `dart run build_runner build --delete-conflicting-outputs`
3. Hot reload/restart the app

### Cross-Platform Development

1. Make changes to shared code (domain, presentation layers)
2. Regenerate code: `dart run build_runner build --delete-conflicting-outputs`
3. Test on both platforms

## Configuration

### Environment Configuration

The app supports multiple environments. Configure in `lib/core/config/app_config.dart`:

```dart
static const Environment env = Environment.dev; // or staging, prod
static bool get isDebugMode => env == Environment.dev;
```

### Platform-Specific Configuration

-   **Desktop**: Rust core configuration in `native/t3chat_core/`
-   **Mobile**: Android configuration in `android/`, iOS in `ios/`

## Upgrading Flutter and Dependencies

### Upgrading Flutter SDK

1. **Check current Flutter version**:

    ```bash
    flutter --version
    ```

2. **Upgrade Flutter**:

    ```bash
    flutter upgrade
    ```

3. **Verify upgrade**:

    ```bash
    flutter doctor
    ```

4. **Update project constraints** (if needed):
    - Check `pubspec.yaml` for SDK constraints
    - Update `environment.sdk` if Flutter requires a newer Dart version

### Upgrading Dependencies

#### Minor and Patch Updates

For minor and patch updates (e.g., `^3.1.0` → `^3.2.0`):

```bash
# Update all dependencies to latest compatible versions
flutter pub upgrade

# Or update specific package
flutter pub upgrade <package_name>
```

#### Major Version Upgrades

For major version upgrades (e.g., `^3.1.0` → `^4.0.0`), follow these steps:

1. **Review breaking changes**:

    - Check the package's changelog/CHANGELOG.md
    - Review migration guides on pub.dev
    - Check GitHub releases for breaking changes

2. **Update pubspec.yaml**:

    ```yaml
    dependencies:
        flutter_riverpod: ^4.0.0 # Major version upgrade
    ```

3. **Update dependencies**:

    ```bash
    flutter pub get
    ```

4. **Fix breaking changes**:

    - Update imports if package structure changed
    - Update API calls if method signatures changed
    - Update provider definitions if Riverpod API changed
    - Check for deprecated APIs and replace them

5. **Regenerate code**:

    ```bash
    dart run build_runner clean
    dart run build_runner build --delete-conflicting-outputs
    ```

6. **Test thoroughly**:
    - Run on all target platforms
    - Test authentication flow
    - Test chat functionality
    - Check for runtime errors

#### Common Major Version Upgrades

##### Riverpod (3.x → 4.x)

-   Update `flutter_riverpod` and `riverpod_annotation` together
-   Review [Riverpod migration guide](https://riverpod.dev/docs/migration/from_state_notifier)
-   Update provider definitions if using code generation
-   Check for API changes in `StateNotifier`, `FutureProvider`, etc.

##### Go Router (17.x → 18.x)

-   Review [go_router changelog](https://pub.dev/packages/go_router/changelog)
-   Update route definitions if API changed
-   Check for navigation API changes

##### Dio (5.x → 6.x)

-   Review [Dio changelog](https://pub.dev/packages/dio/changelog)
-   Update interceptors if API changed
-   Check for response handling changes

##### Flutter Rust Bridge

-   Check [flutter_rust_bridge releases](https://github.com/fzyzcjy/flutter_rust_bridge/releases)
-   Regenerate bridge bindings after upgrade
-   Rebuild Rust core: `cd native/t3chat_core && cargo build`
-   Update FFI call sites if bridge API changed

### Dependency Upgrade Checklist

-   [ ] Review changelog for breaking changes
-   [ ] Update `pubspec.yaml` with new version
-   [ ] Run `flutter pub get`
-   [ ] Fix compilation errors
-   [ ] Update deprecated APIs
-   [ ] Regenerate code (build_runner, rust bridge)
-   [ ] Test on all platforms
-   [ ] Update documentation if needed

### Handling Breaking Changes

1. **Create a feature branch**:

    ```bash
    git checkout -b upgrade/dependency-name-vX.X.X
    ```

2. **Update dependencies incrementally**:

    - Upgrade one major dependency at a time
    - Test after each upgrade
    - Commit working state before next upgrade

3. **Use version constraints wisely**:

    ```yaml
    # Allow patch and minor updates, but require manual review for major
    dependencies:
      package_name: ^3.1.0  # Allows 3.1.0 to <4.0.0

    # For major version upgrades, explicitly update:
    dependencies:
      package_name: ^4.0.0
    ```

4. **Check compatibility**:
    - Ensure all dependencies are compatible with each other
    - Check for dependency conflicts: `flutter pub deps`
    - Resolve conflicts by updating related packages

### Platform-Specific Upgrade Considerations

#### Desktop (Rust FFI)

-   **Flutter Rust Bridge**: Upgrade Rust bridge and Dart package together
-   **Rust dependencies**: Update `native/t3chat_core/Cargo.toml` separately
-   **Rebuild required**: Always rebuild Rust core after bridge upgrade

#### Updating Rust Dependencies

The Rust core (`native/t3chat_core/`) has its own dependency management via Cargo.

**Checking for outdated dependencies:**

```bash
cd native/t3chat_core
cargo outdated
```

**Updating dependencies:**

1. **Minor and patch updates** (e.g., `1.2.0` → `1.2.5`):

    ```bash
    cd native/t3chat_core
    cargo update
    ```

2. **Major version upgrades** (e.g., `1.2.0` → `2.0.0`):

    ```bash
    # 1. Update Cargo.toml with new version
    # Edit native/t3chat_core/Cargo.toml

    # 2. Update dependencies
    cd native/t3chat_core
    cargo update

    # 3. Build to check for breaking changes
    cargo build

    # 4. Fix any compilation errors
    # Update code to match new API

    # 5. Rebuild and regenerate bridge
    cargo build --release
    cd ../..
    flutter_rust_bridge_codegen generate --config-file native/t3chat_core/flutter_rust_bridge.yaml
    ```

**Common Rust dependency upgrades:**

##### Flutter Rust Bridge

-   Check [flutter_rust_bridge releases](https://github.com/fzyzcjy/flutter_rust_bridge/releases)
-   Update both `Cargo.toml` (Rust) and `pubspec.yaml` (Dart) versions together
-   Regenerate bridge bindings after upgrade
-   Review migration guides for API changes

##### Reqwest (HTTP client)

-   Check [reqwest changelog](https://github.com/seanmonstar/reqwest/blob/main/CHANGELOG.md)
-   Major versions may change async runtime requirements
-   Update `tokio` if required by new reqwest version

##### Diesel (Database ORM)

-   Check [Diesel changelog](https://github.com/diesel-rs/diesel/blob/master/CHANGELOG.md)
-   Major versions may require migration syntax changes
-   Update `diesel_cli` if using migrations: `cargo install diesel_cli --no-default-features --features postgres`

##### Keyring (Token storage)

-   Check [keyring-rs releases](https://github.com/hwchen/keyring-rs/releases)
-   Platform-specific behavior may change
-   Test token storage on all target platforms after upgrade

**Rust dependency upgrade checklist:**

-   [ ] Review changelog for breaking changes
-   [ ] Update `Cargo.toml` with new version
-   [ ] Run `cargo update`
-   [ ] Build Rust core: `cargo build`
-   [ ] Fix compilation errors
-   [ ] Test on all desktop platforms (Windows/Linux/macOS)
-   [ ] Rebuild release: `cargo build --release`
-   [ ] Regenerate Flutter Rust Bridge bindings
-   [ ] Test Flutter app with new Rust core
-   [ ] Update documentation if needed

**Handling Rust breaking changes:**

1. **Create a feature branch**:

    ```bash
    git checkout -b upgrade/rust-dependency-vX.X.X
    ```

2. **Update incrementally**:

    - Upgrade one major dependency at a time
    - Test after each upgrade
    - Commit working state before next upgrade

3. **Check compatibility**:

    ```bash
    cd native/t3chat_core
    cargo tree  # View dependency tree
    cargo check # Check for conflicts
    ```

4. **Clean and rebuild**:
    ```bash
    cargo clean
    cargo build --release
    ```

**Rollback Rust dependencies:**

If an upgrade causes issues:

```bash
cd native/t3chat_core
# Revert Cargo.toml
git checkout HEAD -- Cargo.toml
# Revert Cargo.lock
git checkout HEAD -- Cargo.lock
# Or manually edit Cargo.toml and run:
cargo update
cargo build --release
```

#### Mobile

-   **Android**: May need to update `android/build.gradle.kts` for new Gradle versions
-   **iOS**: May need to update `ios/Podfile` and run `pod install`

### Staying Up-to-Date

1. **Regular updates**:

    ```bash
    # Check for outdated packages
    flutter pub outdated

    # Update to latest compatible versions
    flutter pub upgrade
    ```

2. **Security updates**:

    - Monitor security advisories for dependencies
    - Use `flutter pub upgrade` regularly for security patches

3. **Version pinning** (for production):

    ```yaml
    # Pin exact versions for production stability
    dependencies:
        package_name: 3.1.0 # Exact version
    ```

4. **Version ranges** (for development):
    ```yaml
    # Use ranges for flexibility during development
    dependencies:
        package_name: ^3.1.0 # Allows compatible updates
    ```

### Rollback Strategy

If an upgrade causes issues:

1. **Revert pubspec.yaml**:

    ```bash
    git checkout HEAD -- pubspec.yaml
    flutter pub get
    ```

2. **Clean and rebuild**:

    ```bash
    flutter clean
    flutter pub get
    dart run build_runner clean
    dart run build_runner build --delete-conflicting-outputs
    ```

3. **For Rust bridge issues**:
    ```bash
    cd native/t3chat_core
    cargo clean
    cargo build
    ```

## Troubleshooting

### Build Runner Issues

```bash
# Clean and regenerate
dart run build_runner clean
dart run build_runner build --delete-conflicting-outputs
```

### Dependency Resolution Issues

```bash
# Clean and retry
rm -rf .dart_tool
rm pubspec.lock
flutter pub get
```

### Rust Bridge Generation Issues

-   Ensure `flutter_rust_bridge_codegen` is in your PATH
-   Check that Rust core builds successfully: `cd native/t3chat_core && cargo build`
-   Verify `flutter_rust_bridge.yaml` configuration

### Android Build Issues

-   Ensure Android SDK is properly installed
-   Check `android/local.properties` for correct SDK path
-   Run `flutter doctor` to verify setup
-   Update package names in `android/app/build.gradle.kts` if needed

### iOS Build Issues (macOS only)

-   Ensure Xcode is installed and updated
-   Run `pod install` in `ios/` directory
-   Check code signing settings in Xcode

### Network Issues

-   **Android emulator**: Use `10.0.2.2` instead of `localhost` to access host machine
-   **iOS simulator**: `localhost` should work
-   **Desktop**: `localhost` should work
-   Update API base URL in `lib/core/config/app_config.dart` accordingly

### Platform Detection Issues

If the app selects the wrong implementation:

1. Check platform detection logic in `lib/core/providers/repository_providers.dart`
2. Verify `dart:io` Platform imports are correct
3. Ensure `kIsWeb` from `package:flutter/foundation.dart` is used for web detection

## Testing Checklist

-   [ ] Desktop app builds and runs (Windows/Linux/macOS)
-   [ ] Mobile app builds and runs (Android/iOS)
-   [ ] Authentication works on both platforms
-   [ ] Chat list and detail screens work on both
-   [ ] Message streaming works on both
-   [ ] Platform detection correctly selects implementation
-   [ ] All imports resolve correctly
-   [ ] Code generation completes successfully

## Next Steps

1. **Generate Flutter Rust Bridge Bindings** (desktop only)
2. **Update Repository Implementations** after bridge generation
3. **Generate Dart Code** for JSON serialization
4. **Platform-Specific Setup** for each target platform
5. **Testing** on all target platforms

## Architecture Decision

The merged codebase uses platform detection to select the appropriate data layer:

-   **Desktop**: Rust FFI provides better performance and native integration
-   **Mobile**: Dio HTTP is more standard for mobile apps and doesn't require Rust toolchain

This approach allows:

-   Single codebase for all platforms
-   Platform-optimized implementations
-   Shared domain and presentation layers
-   Easier maintenance and feature parity
