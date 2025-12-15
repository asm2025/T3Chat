use anyhow::Result;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct SessionManager {
    secret: String,
    expiry_seconds: u64,
}

impl SessionManager {
    pub fn new(secret: String, expiry_seconds: u64) -> Self {
        Self {
            secret,
            expiry_seconds,
        }
    }

    pub fn generate_token(&self, user: &User) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let claims = SessionClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            exp: now + self.expiry_seconds as i64,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<SessionClaims> {
        use jsonwebtoken::{DecodingKey, Validation, decode, Algorithm};

        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0; // Strict expiration checking
        
        let token_data = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::User;
    use chrono::Utc;
    use std::thread;
    use std::time::Duration;

    fn create_dummy_user() -> User {
        User {
            id: "user123".to_string(),
            email: "test@example.com".to_string(),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
            username: None,
            avatar_url: None,
            provider: "local".to_string(),
            normalized_email: "test@example.com".to_string(),
            normalized_username: None,
            password_hash: None,
            two_factor_enabled: Some(false),
            totp_secret: None,
            disabled: false,
            locked_out: false,
            lockout_end: None,
            access_failed_count: 0,
            is_system: false,
            password_changed_at: None,
            last_login_at: None,
            login_count: 0,
            terms_accepted: Some(true),
            terms_accepted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_generate_and_verify_token() {
        let secret = "test_secret_key_must_be_long_enough".to_string();
        let manager = SessionManager::new(secret, 3600);
        let user = create_dummy_user();

        let token = manager.generate_token(&user).expect("Failed to generate token");
        let claims = manager.verify_token(&token).expect("Failed to verify token");

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_expired_token() {
        let secret = "test_secret".to_string();
        // 1 second expiry
        let manager = SessionManager::new(secret, 1);
        let user = create_dummy_user();

        let token = manager.generate_token(&user).expect("Failed to generate token");
        
        // Wait for 2 seconds
        thread::sleep(Duration::from_secs(2));

        let result = manager.verify_token(&token);
        assert!(result.is_err());
    }
}
