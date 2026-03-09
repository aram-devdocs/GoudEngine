use crate::ecs::components::AudioChannel;

#[test]
fn test_audio_channel_from_id() {
    assert_eq!(AudioChannel::from_id(0), AudioChannel::Music);
    assert_eq!(AudioChannel::from_id(1), AudioChannel::SFX);
    assert_eq!(AudioChannel::from_id(2), AudioChannel::Voice);
    assert_eq!(AudioChannel::from_id(3), AudioChannel::Ambience);
    assert_eq!(AudioChannel::from_id(4), AudioChannel::UI);
    assert_eq!(AudioChannel::from_id(5), AudioChannel::Custom(5));
    assert_eq!(AudioChannel::from_id(255), AudioChannel::Custom(255));
    // Roundtrip: from_id(id).id() == id
    for id in 0..=10 {
        assert_eq!(AudioChannel::from_id(id).id(), id);
    }
}
