use crate::{Error, Result};

pub struct Keychain {
    service: String,
}

impl Keychain {
    pub fn new(service: &str) -> Self {
        Self { service: service.to_string() }
    }

    fn entry(&self, account: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service, account).map_err(Error::from)
    }

    pub fn set(&self, account: &str, secret: &str) -> Result<()> {
        self.entry(account)?.set_password(secret).map_err(Error::from)
    }

    pub fn get(&self, account: &str) -> Result<String> {
        self.entry(account)?.get_password().map_err(Error::from)
    }

    pub fn delete(&self, account: &str) -> Result<()> {
        self.entry(account)?.delete_password().map_err(Error::from)
    }
}
