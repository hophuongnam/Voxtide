use voxtide_core::latency::LatencyTracker;

#[test]
fn median_is_none_until_first_observation() {
    let t = LatencyTracker::new(32);
    assert_eq!(t.median_ms(), None);
}

#[test]
fn median_matches_input_for_known_window() {
    let mut t = LatencyTracker::new(32);
    for v in [100u64, 200, 300, 400, 500] { t.observe(v); }
    assert_eq!(t.median_ms(), Some(300));
}

#[test]
fn rolling_window_caps_at_capacity() {
    let mut t = LatencyTracker::new(3);
    for v in [10u64, 20, 30, 40, 50] { t.observe(v); }
    // last 3 = [30, 40, 50] → median 40
    assert_eq!(t.median_ms(), Some(40));
}
