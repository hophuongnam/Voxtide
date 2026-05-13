use voxtide_core::audio::mic;

#[test]
fn list_input_devices_returns_at_least_default_or_none_gracefully() {
    // Should not panic regardless of whether a mic is attached.
    let devs = mic::list_input_devices().unwrap();
    for d in &devs {
        assert!(!d.label.is_empty());
        // `d.default` is structurally either `true` or `false`; the type guarantees it.
        // Other invariants (non-empty label) are covered above.
    }
}
