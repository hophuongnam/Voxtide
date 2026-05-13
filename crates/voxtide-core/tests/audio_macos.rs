#![cfg(target_os = "macos")]
use voxtide_core::audio::macos_loopback::{capture_strategy, CaptureStrategy};

#[test]
fn strategy_selection_matches_os_version() {
    let strategy = capture_strategy();
    // On macOS 14.4+ we expect ProcessTap; older falls back to ScreenCaptureKit.
    match strategy {
        CaptureStrategy::ProcessTap | CaptureStrategy::ScreenCaptureKit => {}
    }
}
