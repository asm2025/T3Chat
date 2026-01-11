# T3Chat Mobile Client

Flutter mobile client for T3Chat, built according to the architecture plan.

## Architecture

- **State Management**: Riverpod
- **Navigation**: go_router
- **Networking**: dio with interceptors
- **Architecture Pattern**: Clean Architecture (Domain, Data, Presentation layers)

## Setup

1. Install dependencies:
```bash
flutter pub get
```

2. Generate code (for JSON serialization):
```bash
flutter pub run build_runner build --delete-conflicting-outputs
```

3. Update API base URL in `lib/core/config/app_config.dart` if needed.

4. Run the app:
```bash
flutter run
```

## Project Structure

```
lib/
  core/           # Core infrastructure (config, network, error handling, logging)
  domain/         # Domain models and repository interfaces
  data/           # Data layer (API client, repository implementations)
  features/       # Feature modules (auth, chat)
  shared/         # Shared widgets and utilities
  main.dart       # App entry point
```

## Features

- ✅ Authentication (local username/password, OIDC support)
- ✅ Chat list with pagination
- ✅ Chat detail with message streaming (SSE)
- ✅ Secure token storage
- ✅ Error handling and logging

## Environment Configuration

Update `lib/core/config/app_config.dart` to change the API base URL for different environments.

