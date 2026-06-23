//! Platform-free loopback facade: callers (the Tauri shell) must never need
//! a #[cfg] to list or construct a system-audio source.

// ponytail: import gated to match its only consumers — the macOS tests below.
// On Windows all three tests compile out, so an ungated import is unused →
// `clippy -D warnings` fails on the Windows runner only (invisible to macOS).
#[cfg(target_os = "macos")]
use voxtide_core::audio::loopback;

#[cfg(target_os = "macos")]
#[test]
fn macos_lists_single_synthetic_system_endpoint() {
    let devices = loopback::list().expect("list");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id, "system");
    assert!(!devices[0].label.is_empty());
    // Pre-facade behavior preserved: macOS never marked the entry default.
    assert!(!devices[0].default);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_by_id_resolves_the_listed_endpoint() {
    let source = loopback::by_id("system").expect("by_id");
    assert!(!source.label().is_empty());
}

#[cfg(target_os = "macos")]
#[test]
fn unknown_id_error_mentions_not_found() {
    // "not found" is LOAD-BEARING: the shell's StartError::classify maps it
    // to `device-missing` (plain error strip) instead of a permission banner.
    let err = match loopback::by_id("nope") {
        Ok(_) => panic!("unknown id must fail"),
        Err(e) => e,
    };
    assert!(
        err.to_string().contains("not found"),
        "error must contain 'not found', got: {err}"
    );
}
