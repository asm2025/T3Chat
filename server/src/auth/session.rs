use anyhow::Result;
use jsonwebtoken::{encode, EncodingKey, Header};
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
        use jsonwebtoken::{decode, DecodingKey, Validation};

        let validation = Validation::default();
        let token_data = decode::<SessionClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
}
