use voxtide_core::keychain::Keychain;

#[test]
fn round_trip_api_key_or_skip() {
    let kc = Keychain::new("voxtide-test");
    match kc.set("alice@example.com", "sk_live_abc") {
        Ok(()) => {
            let got = kc.get("alice@example.com").unwrap();
            assert_eq!(got, "sk_live_abc");
            kc.delete("alice@example.com").unwrap();
            assert!(kc.get("alice@example.com").is_err());
        }
        Err(e) => {
            eprintln!("skipping (no usable keyring): {e}");
        }
    }
}
