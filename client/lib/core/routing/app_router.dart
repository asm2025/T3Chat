import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../features/auth/presentation/screens/login_screen.dart';
import '../../features/auth/presentation/screens/oidc_callback_screen.dart';
import '../../features/chat/list/presentation/screens/chat_list_screen.dart';
import '../../features/chat/detail/presentation/screens/chat_detail_screen.dart';
import '../../features/auth/providers/auth_provider.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  final authState = ref.watch(authStateProvider);

  return GoRouter(
    initialLocation: '/',
    redirect: (context, state) {
      final isLoggedIn = authState.value != null;
      final isLoggingIn = state.uri.path == '/login' || state.uri.path.startsWith('/callback');

      if (!isLoggedIn && !isLoggingIn) {
        return '/login';
      }
      if (isLoggedIn && isLoggingIn) {
        return '/app';
      }
      return null;
    },
    routes: [
      // Explicit root route so `/` can be matched before redirects are evaluated.
      // Without this, go_router shows "no routes for location: /".
      GoRoute(
        path: '/',
        redirect: (context, state) {
          final isLoggedIn = authState.value != null;
          return isLoggedIn ? '/app' : '/login';
        },
      ),
      GoRoute(
        path: '/login',
        builder: (context, state) => const LoginScreen(),
      ),
      GoRoute(
        path: '/callback',
        builder: (context, state) {
          final token = state.uri.queryParameters['token'];
          return OidcCallbackScreen(token: token);
        },
      ),
      GoRoute(
        path: '/app',
        builder: (context, state) => const ChatListScreen(),
        routes: [
          GoRoute(
            path: 'chat/:chatId',
            builder: (context, state) {
              final chatId = state.pathParameters['chatId']!;
              return ChatDetailScreen(chatId: chatId);
            },
          ),
        ],
      ),
    ],
  );
});

