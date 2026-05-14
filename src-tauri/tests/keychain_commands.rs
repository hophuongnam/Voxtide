use tempfile::TempDir;

#[tokio::test]
async fn round_trip_via_file_store() {
    let dir = TempDir::new().unwrap();
    let kc = voxtide_core::Keychain::new(dir.path().join("secrets.json"));
    kc.set("alice", "sk_live_abc").unwrap();
    assert_eq!(kc.get("alice").unwrap(), "sk_live_abc");
    kc.delete("alice").unwrap();
    assert!(kc.get("alice").is_err());
}
