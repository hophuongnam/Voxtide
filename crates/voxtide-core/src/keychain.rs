use crate::{Error, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub struct Keychain {
    path: PathBuf,
}

impl Keychain {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Read the raw store bytes. Missing file = empty store; real IO errors
    /// propagate (they must block writes too — clobbering the store over a
    /// transient EACCES would lose every other account).
    fn read_bytes(&self) -> Result<Vec<u8>> {
        match fs::read(&self.path) {
            Ok(b) => Ok(b),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(Error::Keychain(format!(
                "read {}: {e}",
                self.path.display()
            ))),
        }
    }

    fn parse(path: &Path, bytes: &[u8]) -> Result<BTreeMap<String, String>> {
        if bytes.is_empty() {
            return Ok(BTreeMap::new());
        }
        serde_json::from_slice(bytes)
            .map_err(|e| Error::Keychain(format!("parse {}: {e}", path.display())))
    }

    fn load(&self) -> Result<BTreeMap<String, String>> {
        Self::parse(&self.path, &self.read_bytes()?)
    }

    /// Like [`Self::load`], but a corrupt (unparsable) store is quarantined to
    /// `secrets.json.corrupt` and treated as empty. Used by the WRITE paths
    /// (`set`/`delete`) so a corrupt file can always be healed by writing —
    /// before this, the parse error propagated and even clearing the key
    /// couldn't recover. IO errors still propagate; reads (`get`) keep strict
    /// behavior (the command layer maps their errors without destroying
    /// evidence).
    fn load_or_quarantine(&self) -> Result<BTreeMap<String, String>> {
        let bytes = self.read_bytes()?;
        match Self::parse(&self.path, &bytes) {
            Ok(m) => Ok(m),
            Err(e) => {
                let quarantine = self.path.with_extension("json.corrupt");
                let _ = fs::rename(&self.path, &quarantine);
                tracing::warn!(
                    error = %e,
                    path = %self.path.display(),
                    "corrupt secrets store quarantined; starting empty"
                );
                Ok(BTreeMap::new())
            }
        }
    }

    fn save(&self, map: &BTreeMap<String, String>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Keychain(format!("mkdir {}: {e}", parent.display())))?;
        }
        let bytes = serde_json::to_vec_pretty(map)
            .map_err(|e| Error::Keychain(format!("serialize: {e}")))?;
        let tmp = self.path.with_extension("json.tmp");
        // Remove any stale tmp so the create below always makes a FRESH file:
        // OpenOptions::mode() applies only at creation, and a leftover tmp
        // from a crashed save could carry looser permissions.
        let _ = fs::remove_file(&tmp);
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            // 0600 from the first byte. The old fs::write + chmod-after left
            // a window where the secrets sat umask-readable (typically 0644).
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&tmp)
                .map_err(|e| Error::Keychain(format!("create {}: {e}", tmp.display())))?;
            f.write_all(&bytes)
                .map_err(|e| Error::Keychain(format!("write {}: {e}", tmp.display())))?;
        }
        #[cfg(not(unix))]
        fs::write(&tmp, &bytes)
            .map_err(|e| Error::Keychain(format!("write {}: {e}", tmp.display())))?;
        fs::rename(&tmp, &self.path)
            .map_err(|e| Error::Keychain(format!("rename to {}: {e}", self.path.display())))?;
        Ok(())
    }

    pub fn set(&self, account: &str, secret: &str) -> Result<()> {
        let mut map = self.load_or_quarantine()?;
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
        let mut map = self.load_or_quarantine()?;
        if map.remove(account).is_some() {
            self.save(&map)?;
        }
        Ok(())
    }
}
