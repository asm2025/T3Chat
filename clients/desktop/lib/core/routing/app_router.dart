import 'package:go_router/go_router.dart';
import '../../features/auth/presentation/screens/login_screen.dart';
import '../../features/auth/presentation/screens/oidc_callback_screen.dart';
import '../../features/chat/list/presentation/screens/chat_list_screen.dart';
import '../../features/chat/detail/presentation/screens/chat_detail_screen.dart';

final appRouter = GoRouter(
  initialLocation: '/login',
  routes: [
    GoRoute(
      path: '/login',
      builder: (context, state) => const LoginScreen(),
    ),
    GoRoute(
      path: '/auth/callback',
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

