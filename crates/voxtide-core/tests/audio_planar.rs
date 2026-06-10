use voxtide_core::audio::planar_to_interleaved;

#[test]
fn interleaves_two_planar_channels() {
    let l = vec![1.0f32, 2.0, 3.0];
    let r = vec![10.0f32, 20.0, 30.0];
    assert_eq!(
        planar_to_interleaved(&[l, r]),
        vec![1.0, 10.0, 2.0, 20.0, 3.0, 30.0]
    );
}

#[test]
fn single_buffer_passes_through() {
    let b = vec![1.0f32, 2.0, 3.0, 4.0];
    assert_eq!(planar_to_interleaved(std::slice::from_ref(&b)), b);
}

#[test]
fn empty_input_yields_empty() {
    assert_eq!(planar_to_interleaved(&[]), Vec::<f32>::new());
}

#[test]
fn three_channels_interleave_round_robin() {
    let a = vec![1.0f32, 2.0];
    let b = vec![10.0f32, 20.0];
    let c = vec![100.0f32, 200.0];
    assert_eq!(
        planar_to_interleaved(&[a, b, c]),
        vec![1.0, 10.0, 100.0, 2.0, 20.0, 200.0]
    );
}

#[test]
fn uneven_buffers_truncate_to_shortest() {
    let l = vec![1.0f32, 2.0, 3.0];
    let r = vec![10.0f32, 20.0];
    assert_eq!(planar_to_interleaved(&[l, r]), vec![1.0, 10.0, 2.0, 20.0]);
}
