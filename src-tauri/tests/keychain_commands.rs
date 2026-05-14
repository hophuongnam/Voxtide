#[tokio::test]
async fn round_trip_via_keyring_or_skip() {
    let kc = voxtide_core::Keychain::new("voxtide-tauri-test");
    match kc.set("alice", "sk_live_abc") {
        Ok(()) => {
            assert_eq!(kc.get("alice").unwrap(), "sk_live_abc");
            kc.delete("alice").unwrap();
        }
        Err(e) => eprintln!("skipping: {e}"),
    }
}
