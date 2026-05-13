use voxtide_core::speaker_map::SpeakerMap;

#[test]
fn first_heard_speaker_maps_to_a() {
    let mut m = SpeakerMap::new();
    assert_eq!(m.chip_for("1"), 'A');
    assert_eq!(m.chip_for("2"), 'B');
    assert_eq!(m.chip_for("1"), 'A'); // stable
}

#[test]
fn cycles_at_d() {
    let mut m = SpeakerMap::new();
    for (id, want) in [("1", 'A'), ("2", 'B'), ("3", 'C'), ("4", 'D'), ("5", 'A')] {
        assert_eq!(m.chip_for(id), want);
    }
}

#[test]
fn serializes_to_and_from_map() {
    let mut m = SpeakerMap::new();
    m.chip_for("1"); m.chip_for("2");
    let snapshot = m.snapshot();
    let m2 = SpeakerMap::from_snapshot(snapshot.clone());
    assert_eq!(m2.snapshot(), snapshot);
}
