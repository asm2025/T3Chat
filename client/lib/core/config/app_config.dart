enum Environment { dev, staging, prod }

class AppConfig {
  static const Environment env = Environment.dev;

  static String get apiBaseUrl {
    switch (env) {
      case Environment.dev:
        return 'http://localhost:3000';
      case Environment.staging:
        return 'https://staging-api.example.com';
      case Environment.prod:
        return 'https://api.example.com';
    }
  }

  static bool get isDebugMode => env == Environment.dev;
}

