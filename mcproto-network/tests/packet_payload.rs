//! Tests for typed packet payload decoding and metadata helpers.

use mcproto_network::{
    CompressionMode, DecryptionMode, Direction, Packet, PacketCodec, PacketId, PacketName,
    PacketPayloadError, PacketReadLimits, PacketReader, ProtocolState,
};
use mcproto_types::VarInt;

#[derive(Debug, PartialEq, PacketCodec)]
#[packet(
    name = "payload_value",
    id = 0x2a,
    state = Play,
    direction = Clientbound
)]
struct PayloadValue {
    value: VarInt,
}

fn read_payload(frame: &[u8]) -> mcproto_network::PacketPayload {
    PacketReader::new(
        frame,
        CompressionMode::disabled(),
        DecryptionMode::disabled(),
        PacketReadLimits::default(),
    )
    .read_packet()
    .unwrap()
}

#[test]
fn packet_metadata_helpers_expose_declared_constants() {
    assert_eq!(PayloadValue::name(), PacketName::new("payload_value"));
    assert_eq!(PayloadValue::id(), PacketId::new(0x2a).unwrap());
    assert_eq!(PayloadValue::state(), ProtocolState::Play);
    assert_eq!(PayloadValue::direction(), Direction::Clientbound);
    assert!(PayloadValue::matches_id(PacketId::new(0x2a).unwrap()));

    let info = PayloadValue::info();
    assert_eq!(info.name, PayloadValue::NAME);
    assert_eq!(info.id, PayloadValue::ID);
    assert_eq!(info.state, PayloadValue::STATE);
    assert_eq!(info.direction, PayloadValue::DIRECTION);
}

#[test]
fn packet_payload_decodes_the_matching_type_exactly() {
    let payload = read_payload(&[0x03, 0x2a, 0xac, 0x02]);

    assert_eq!(
        payload.decode::<PayloadValue>().unwrap(),
        PayloadValue { value: VarInt(300) }
    );
}

#[test]
fn packet_payload_rejects_an_id_mismatch() {
    let payload = read_payload(&[0x03, 0x2b, 0xac, 0x02]);

    assert!(matches!(
        payload.decode::<PayloadValue>(),
        Err(PacketPayloadError::PacketIdMismatch {
            expected,
            actual,
        }) if expected.get() == 0x2a && actual.get() == 0x2b
    ));
}

#[test]
fn packet_payload_rejects_unconsumed_trailing_bytes() {
    let payload = read_payload(&[0x04, 0x2a, 0xac, 0x02, 0xaa]);

    assert!(matches!(
        payload.decode::<PayloadValue>(),
        Err(PacketPayloadError::TrailingBytes { remaining: 1, .. })
    ));
}

#[test]
fn packet_payload_preserves_body_codec_errors() {
    let payload = read_payload(&[0x02, 0x2a, 0x80]);

    assert!(matches!(
        payload.decode::<PayloadValue>(),
        Err(PacketPayloadError::Codec { .. })
    ));
}
