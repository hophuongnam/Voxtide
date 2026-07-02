//! Keep the overlay on the display the cursor is on (macOS).
//!
//! `CanJoinAllSpaces | FullScreenAuxiliary` (set at window build) makes the
//! overlay follow Spaces — but only on the display its frame occupies: macOS
//! never shows a window over another display's Space, fullscreen or not. With
//! multiple monitors the overlay was pinned to whichever screen it spawned
//! on, so a fullscreen app on the *other* display never got captions.
//!
//! The cursor's screen is the follow signal (chosen over keyboard focus:
//! focus-following misses the watch-without-clicking case — fullscreen video
//! on the second display never gets focused, but the mouse parks there). A
//! 500 ms main-run-loop timer compares the cursor's screen to the overlay's
//! and hops the window when they differ, preserving fractional placement (a
//! bottom-center overlay lands bottom-center on the next screen too). Ticks
//! are skipped while the left button is down so the poll never teleports the
//! window out from under the hover-strip drag (tao's drag loop owns the
//! frame then); dwelling on the overlay's own screen is a no-op.

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSScreen, NSWindow};
use objc2_foundation::{NSPoint, NSRect, NSTimer};

/// Remap one axis of a window origin from one screen span to another, keeping
/// the window's fractional position within the space it can occupy. A window
/// wider/taller than the span (or exactly filling it) pins to the target
/// origin via the 0.5-fraction / zero-span path.
fn axis_remap(win_o: f64, win_len: f64, from_o: f64, from_len: f64, to_o: f64, to_len: f64) -> f64 {
    let span = from_len - win_len;
    let frac = if span > 0.0 {
        ((win_o - from_o) / span).clamp(0.0, 1.0)
    } else {
        0.5
    };
    to_o + frac * (to_len - win_len).max(0.0)
}

fn rect_eq(a: NSRect, b: NSRect) -> bool {
    a.origin.x == b.origin.x
        && a.origin.y == b.origin.y
        && a.size.width == b.size.width
        && a.size.height == b.size.height
}

/// Right/top-edge-exclusive containment, matching `NSMouseInRect` semantics
/// so a cursor on a shared display edge resolves to exactly one screen.
fn contains(r: NSRect, p: NSPoint) -> bool {
    p.x >= r.origin.x
        && p.x < r.origin.x + r.size.width
        && p.y >= r.origin.y
        && p.y < r.origin.y + r.size.height
}

/// Schedule the app-lifetime follow timer on the main run loop. Best-effort:
/// if the native handle is unavailable the overlay keeps the pinned behavior.
pub fn register(overlay: &tauri::WebviewWindow) {
    let Ok(ptr) = overlay.ns_window() else { return };
    // +1 retain so the block can never dangle. The overlay window lives for
    // the app's lifetime anyway (created once in setup, only shown/hidden).
    let Some(ns): Option<Retained<NSWindow>> = (unsafe { Retained::retain(ptr.cast()) }) else {
        return;
    };
    let block = RcBlock::new(move |_: core::ptr::NonNull<NSTimer>| {
        // Scheduled from setup on the main run loop → main-thread ticks.
        let Some(mtm) = MainThreadMarker::new() else { return };
        // Left button down = the user may be dragging the overlay (or is
        // mid-interaction elsewhere); never move the frame under a drag.
        if NSEvent::pressedMouseButtons() & 1 != 0 {
            return;
        }
        let cursor = NSEvent::mouseLocation();
        let Some(target) = NSScreen::screens(mtm)
            .iter()
            .find(|s| contains(s.frame(), cursor))
        else {
            return;
        };
        let to = target.frame();
        let win = ns.frame();
        // Off-screen (no current screen) degrades to the zero-span path
        // below, which centers the overlay on the target screen.
        let from = ns.screen().map(|s| s.frame()).unwrap_or(win);
        if rect_eq(from, to) {
            return;
        }
        ns.setFrameOrigin(NSPoint {
            x: axis_remap(
                win.origin.x,
                win.size.width,
                from.origin.x,
                from.size.width,
                to.origin.x,
                to.size.width,
            ),
            y: axis_remap(
                win.origin.y,
                win.size.height,
                from.origin.y,
                from.size.height,
                to.origin.y,
                to.size.height,
            ),
        });
    });
    // The main run loop retains a scheduled repeating timer until it is
    // invalidated (never — app-lifetime), so the returned handle can drop.
    let _ = unsafe { NSTimer::scheduledTimerWithTimeInterval_repeats_block(0.5, true, &block) };
}

#[cfg(test)]
mod tests {
    use super::{axis_remap, contains};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    #[test]
    fn preserves_fraction_across_different_spans() {
        // 25% into a 1000-wide screen (span 1000-100=900, origin 0) ...
        let x = axis_remap(225.0, 100.0, 0.0, 1000.0, 2000.0, 500.0);
        // ... lands 25% into the 500-wide screen at origin 2000 (span 400).
        assert_eq!(x, 2000.0 + 0.25 * 400.0);
    }

    #[test]
    fn edges_stay_edges() {
        assert_eq!(axis_remap(0.0, 100.0, 0.0, 1000.0, 500.0, 800.0), 500.0);
        assert_eq!(
            axis_remap(900.0, 100.0, 0.0, 1000.0, 500.0, 800.0),
            500.0 + 700.0
        );
    }

    #[test]
    fn oversized_window_pins_to_target_origin() {
        // Window wider than the source span → frac 0.5; wider than the target
        // too → max(0) span → exactly the target origin.
        assert_eq!(axis_remap(0.0, 2000.0, 0.0, 1000.0, 300.0, 1500.0), 300.0);
    }

    #[test]
    fn containment_is_right_top_exclusive() {
        let r = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 100.0,
                height: 50.0,
            },
        };
        assert!(contains(r, NSPoint { x: 0.0, y: 0.0 }));
        assert!(contains(r, NSPoint { x: 99.9, y: 49.9 }));
        assert!(!contains(r, NSPoint { x: 100.0, y: 25.0 }));
        assert!(!contains(r, NSPoint { x: 50.0, y: 50.0 }));
    }
}
