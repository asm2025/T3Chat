# OIDC Authentication Module

This module provides a clean and maintainable OpenID Connect (OIDC) authentication implementation for the server.

## Features

- **Automatic Provider Discovery**: Automatically discovers OIDC provider configuration from the `.well-known/openid-configuration` endpoint
- **Token Management**: Handles authorization code exchange and token refresh
- **User Information Retrieval**: Fetches user profile information from the OIDC provider
- **Error Handling**: Comprehensive error handling with context
- **Async/Await**: Fully asynchronous implementation using Tokio
- **Thread-Safe**: Uses RwLock for safe concurrent access to metadata

## Usage

### Initialization

```rust
use crate::auth::OidcClient;

let oidc_client = OidcClient::new(
    "https://accounts.google.com".to_string(), // Issuer URL
    "your-client-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/api/v1/auth/callback".to_string(),
).await?;
```

### Authorization Flow

1. **Generate Authorization URL**:

```rust
let (auth_url, state_token) = oidc_client.get_authorization_url();
// Redirect user to auth_url
// Store state_token in session for verification
```

2. **Handle Callback and Exchange Code**:

```rust
// After user is redirected back with code and state
let token_response = oidc_client.exchange_code(code, state).await?;

// token_response contains:
// - access_token: String
// - refresh_token: Option<String>
// - expires_in: Option<u64>
// - id_token: Option<String>
```

3. **Get User Information**:

```rust
let user_info = oidc_client.get_user_info(token_response.access_token).await?;

// user_info contains:
// - sub: User ID
// - email: Email address
// - email_verified: Email verification status
// - name, given_name, family_name: User names
// - picture: Profile picture URL
// - preferred_username: Username
```

4. **Refresh Token**:

```rust
if let Some(refresh_token) = token_response.refresh_token {
    let new_tokens = oidc_client.refresh_token(refresh_token).await?;
    // Use new_tokens.access_token
}
```

## Key Improvements Over Previous Implementation

1. **Simpler Dependencies**: Uses standard `reqwest` instead of the heavy `openidconnect` crate
2. **Better Error Messages**: Detailed error context for debugging
3. **Cleaner Code**: More straightforward implementation without unnecessary abstractions
4. **Async RwLock**: Thread-safe metadata caching with minimal locking
5. **Logging**: Comprehensive logging at debug and info levels
6. **Less Memory**: Smaller footprint and fewer dependencies

## Supported OIDC Providers

This implementation follows the OpenID Connect Discovery 1.0 specification and should work with any compliant provider:

- Google
- Microsoft Azure AD
- Auth0
- Keycloak
- Okta
- And many more...

## Environment Variables

Make sure to set these environment variables:

```bash
OIDC_ISSUER_URL=https://accounts.google.com
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/api/v1/auth/callback
```

## Security Considerations

1. **State Token**: Always verify the state token in callbacks to prevent CSRF attacks
2. **HTTPS Only**: Use HTTPS in production for redirect URIs
3. **Secure Storage**: Store refresh tokens securely (encrypted in database)
4. **Token Expiration**: Check and handle token expiration appropriately
5. **Scope Limitations**: Only request the scopes you need (openid, email, profile)

## Testing

The module includes unit tests for token generation:

```bash
cargo test --package server --lib auth::oidc
```

## Troubleshooting

### Provider Discovery Fails

- Check that `OIDC_ISSUER_URL` is correct and accessible
- Ensure the provider supports the `.well-known/openid-configuration` endpoint
- Check network connectivity and firewall rules

### Token Exchange Fails

- Verify client ID and secret are correct
- Ensure redirect URI matches exactly (including trailing slashes)
- Check that the authorization code hasn't expired (typically 10 minutes)

### User Info Request Fails

- Verify the access token is valid and not expired
- Check that the required scopes were granted (openid, email, profile)
- Ensure the provider's userinfo endpoint is accessible

