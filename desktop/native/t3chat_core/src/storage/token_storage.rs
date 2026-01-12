use crate::domain::Error;
use keyring::Entry;

const SERVICE_NAME: &str = "t3chat";
const TOKEN_KEY: &str = "access_token";

pub struct TokenStorage;

impl TokenStorage {
    pub fn new() -> Self {
        Self
    }

    pub async fn save_token(&self, token: &str) -> Result<(), Error> {
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
        entry.set_password(token)?;
        Ok(())
    }

    pub async fn get_token(&self) -> Result<Option<String>, Error> {
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    pub async fn delete_token(&self) -> Result<(), Error> {
        let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(Error::from(e)),
        }
    }
}

