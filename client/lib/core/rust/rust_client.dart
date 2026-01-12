// ignore_for_file: invalid_use_of_internal_member

import '../config/app_config.dart';
import 'bridge_generated.dart/frb_generated.dart' as frb;

class RustClient {
  static bool _initialized = false;

  static Future<void> initialize({String? baseUrl}) async {
    if (_initialized) return;

    final url = baseUrl ?? AppConfig.apiBaseUrl;

    // 1) Initialize flutter_rust_bridge runtime + load the dynamic library.
    await frb.RustLib.init();

    // 2) Initialize the Rust-side API client with the base URL.
    // This is defined in `native/t3chat_core/src/ffi/bridge.rs` as `init(base_url: String)`.
    frb.RustLib.instance.api.crateFfiBridgeInit(baseUrl: url);

    _initialized = true;
  }
}

