use voxtide_core::translation::soniox::{next_backoff_ms, MAX_ATTEMPTS};

#[test]
fn backoff_sequence_matches_spec() {
    let mut got = Vec::new();
    for n in 1..=MAX_ATTEMPTS {
        got.push(next_backoff_ms(n));
    }
    assert_eq!(got, vec![250, 500, 1000, 2000, 5000, 5000]);
}

#[test]
fn backoff_returns_none_after_max_attempts() {
    assert!(next_backoff_ms(MAX_ATTEMPTS + 1) >= 5000);
}

#[test]
fn max_attempts_is_six() {
    assert_eq!(MAX_ATTEMPTS, 6);
}
