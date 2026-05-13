#[test]
fn crate_compiles_and_exposes_version() {
    assert!(!voxtide_core::VERSION.is_empty());
}
