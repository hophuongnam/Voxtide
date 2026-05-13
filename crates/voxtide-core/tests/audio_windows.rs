#![cfg(target_os = "windows")]
use voxtide_core::audio::windows_loopback;

#[test]
fn list_render_endpoints_does_not_panic() {
    let _ = windows_loopback::list_loopback_sources();
}
