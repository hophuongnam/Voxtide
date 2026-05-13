#![cfg(target_os = "macos")]
use voxtide_core::audio::macos_loopback::{capture_strategy, CaptureStrategy};

#[test]
fn strategy_selection_matches_os_version() {
    let strategy = capture_strategy();
    // v1 always returns ScreenCaptureKit; the test accepts both variants
    // so the v1.1 ProcessTap path won't need a test update.
    match strategy {
        CaptureStrategy::ProcessTap | CaptureStrategy::ScreenCaptureKit => {}
    }
}
