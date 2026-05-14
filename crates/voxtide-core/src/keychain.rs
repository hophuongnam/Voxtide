use crate::{Error, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

pub struct Keychain {
    path: PathBuf,
}

impl Keychain {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn load(&self) -> Result<BTreeMap<String, String>> {
        let bytes = match fs::read(&self.path) {
            Ok(b) => b,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => {
                return Err(Error::Keychain(format!(
                    "read {}: {e}",
                    self.path.display()
                )))
            }
        };
        if bytes.is_empty() {
            return Ok(BTreeMap::new());
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| Error::Keychain(format!("parse {}: {e}", self.path.display())))
    }

    fn save(&self, map: &BTreeMap<String, String>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Keychain(format!("mkdir {}: {e}", parent.display())))?;
        }
        let bytes = serde_json::to_vec_pretty(map)
            .map_err(|e| Error::Keychain(format!("serialize: {e}")))?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, &bytes)
            .map_err(|e| Error::Keychain(format!("write {}: {e}", tmp.display())))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::Keychain(format!("chmod {}: {e}", tmp.display())))?;
        }
        fs::rename(&tmp, &self.path)
            .map_err(|e| Error::Keychain(format!("rename to {}: {e}", self.path.display())))?;
        Ok(())
    }

    pub fn set(&self, account: &str, secret: &str) -> Result<()> {
        let mut map = self.load()?;
        map.insert(account.to_string(), secret.to_string());
        self.save(&map)
    }

    pub fn get(&self, account: &str) -> Result<String> {
        let map = self.load()?;
        map.get(account)
            .cloned()
            .ok_or_else(|| Error::Keychain(format!("no secret for account '{account}'")))
    }

    pub fn delete(&self, account: &str) -> Result<()> {
        let mut map = self.load()?;
        if map.remove(account).is_some() {
            self.save(&map)?;
        }
        Ok(())
    }
}
