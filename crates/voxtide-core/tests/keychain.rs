use tempfile::TempDir;
use voxtide_core::keychain::Keychain;

#[test]
fn round_trip_set_get_delete() {
    let dir = TempDir::new().unwrap();
    let kc = Keychain::new(dir.path().join("secrets.json"));

    kc.set("alice@example.com", "sk_live_abc").unwrap();
    assert_eq!(kc.get("alice@example.com").unwrap(), "sk_live_abc");

    kc.delete("alice@example.com").unwrap();
    assert!(kc.get("alice@example.com").is_err());
}

#[test]
fn overwrite_existing_account() {
    let dir = TempDir::new().unwrap();
    let kc = Keychain::new(dir.path().join("secrets.json"));

    kc.set("default", "first").unwrap();
    kc.set("default", "second").unwrap();
    assert_eq!(kc.get("default").unwrap(), "second");
}

#[test]
fn missing_account_returns_err() {
    let dir = TempDir::new().unwrap();
    let kc = Keychain::new(dir.path().join("secrets.json"));
    assert!(kc.get("never-set").is_err());
}

#[test]
fn multiple_accounts_coexist() {
    let dir = TempDir::new().unwrap();
    let kc = Keychain::new(dir.path().join("secrets.json"));

    kc.set("a", "1").unwrap();
    kc.set("b", "2").unwrap();
    assert_eq!(kc.get("a").unwrap(), "1");
    assert_eq!(kc.get("b").unwrap(), "2");

    kc.delete("a").unwrap();
    assert!(kc.get("a").is_err());
    assert_eq!(kc.get("b").unwrap(), "2");
}

#[test]
fn delete_nonexistent_is_ok() {
    let dir = TempDir::new().unwrap();
    let kc = Keychain::new(dir.path().join("secrets.json"));
    kc.delete("nope").unwrap();
}

#[cfg(unix)]
#[test]
fn file_is_user_only() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("secrets.json");
    let kc = Keychain::new(&path);
    kc.set("default", "secret").unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "secrets.json must be 0600, got {:o}", mode);
}
