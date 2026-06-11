use voxtide_core::speaker_map::SpeakerMap;

#[test]
fn first_heard_speaker_maps_to_a() {
    let mut m = SpeakerMap::new();
    assert_eq!(m.chip_for("1"), 'A');
    assert_eq!(m.chip_for("2"), 'B');
    assert_eq!(m.chip_for("1"), 'A'); // stable
}

#[test]
fn five_distinct_speakers_get_five_distinct_letters() {
    // The old 4-letter table wrapped the 5th speaker onto 'A' — chip-equality
    // merging then fused two different people, and only the wrapped letter
    // was persisted (unrecoverable).
    let mut m = SpeakerMap::new();
    for (id, want) in [("1", 'A'), ("2", 'B'), ("3", 'C'), ("4", 'D'), ("5", 'E')] {
        assert_eq!(m.chip_for(id), want);
    }
}

#[test]
fn twenty_six_letters_before_pathological_wrap() {
    let mut m = SpeakerMap::new();
    let mut letters = std::collections::BTreeSet::new();
    for i in 0..26 {
        letters.insert(m.chip_for(&format!("spk{i}")));
    }
    assert_eq!(letters.len(), 26, "26 distinct chips before any wrap");
    // The 27th speaker (pathological) reuses 'A' rather than panicking.
    assert_eq!(m.chip_for("spk26"), 'A');
}

#[test]
fn serializes_to_and_from_map() {
    let mut m = SpeakerMap::new();
    m.chip_for("1");
    m.chip_for("2");
    let snapshot = m.snapshot();
    let m2 = SpeakerMap::from_snapshot(snapshot.clone());
    assert_eq!(m2.snapshot(), snapshot);
}
