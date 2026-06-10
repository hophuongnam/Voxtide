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
    assert_eq!(planar_to_interleaved(&[b.clone()]), b);
}

#[test]
fn uneven_buffers_truncate_to_shortest() {
    let l = vec![1.0f32, 2.0, 3.0];
    let r = vec![10.0f32, 20.0];
    assert_eq!(planar_to_interleaved(&[l, r]), vec![1.0, 10.0, 2.0, 20.0]);
}
