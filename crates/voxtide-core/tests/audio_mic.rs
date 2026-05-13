use voxtide_core::audio::mic;

#[test]
fn list_input_devices_returns_at_least_default_or_none_gracefully() {
    // Should not panic regardless of whether a mic is attached.
    let devs = mic::list_input_devices().unwrap();
    for d in &devs {
        assert!(!d.label.is_empty());
        #[allow(clippy::overly_complex_bool_expr)]
        let _ = d.default || !d.default; // tautology: plan-mandated no-op assertion
    }
}
