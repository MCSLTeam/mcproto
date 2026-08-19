//! Integration tests for the `PacketCodec` derive.

use mcproto_network::{
    CompressionMode, DecodePacket, Direction, EncodePacket, EncryptionMode, Packet, PacketCodec,
    PacketEncoder, PacketId, PacketLimits, PacketName, ProtocolState,
};
use mcproto_types::{Int, UnsignedByte};

#[derive(Debug, PartialEq, PacketCodec)]
#[packet(
    name = "named_test",
    id = 0x2a,
    state = Configuration,
    direction = Clientbound
)]
struct NamedPacket {
    first: UnsignedByte,
    second: Int,
}

#[derive(Debug, PartialEq, PacketCodec)]
#[packet(
    name = "generic_test",
    id = 0x02,
    state = Play,
    direction = Serverbound
)]
struct GenericPacket<T>(UnsignedByte, T);

#[test]
fn packet_names_require_lower_case() {
    assert_eq!(PacketName::new("status_request").as_str(), "status_request");
    assert!(std::panic::catch_unwind(|| PacketName::new("StatusRequest")).is_err());
    assert!(std::panic::catch_unwind(|| PacketName::new("minecraft:status_request")).is_err());
    assert!(std::panic::catch_unwind(|| PacketName::new("status2_request")).is_err());
    assert!(std::panic::catch_unwind(|| PacketName::new("_status_request")).is_err());
    assert!(std::panic::catch_unwind(|| PacketName::new("status__request")).is_err());
    assert!(std::panic::catch_unwind(|| PacketName::new("status_request_")).is_err());
}

#[test]
fn derives_field_ordered_clientbound_decoder() {
    let mut input = [0xaa, 0x01, 0x02, 0x03, 0x04].as_slice();
    assert_eq!(
        NamedPacket::decode_body(&mut input).unwrap(),
        NamedPacket {
            first: UnsignedByte(0xaa),
            second: Int(0x0102_0304),
        }
    );
    assert!(input.is_empty());
}

#[test]
fn derives_named_packet_metadata() {
    assert_eq!(NamedPacket::NAME, PacketName::new("named_test"));
    assert_eq!(NamedPacket::ID, PacketId::new(0x2a).unwrap());
    assert_eq!(NamedPacket::STATE, ProtocolState::Configuration);
    assert_eq!(NamedPacket::DIRECTION, Direction::Clientbound);
}

#[test]
fn default_method_encodes_declared_packet_id() {
    let mut encoder = PacketEncoder::new(
        CompressionMode::disabled(),
        EncryptionMode::disabled(),
        PacketLimits::default(),
    );
    let packet = GenericPacket(UnsignedByte(0xaa), Int(1));

    let mut body = Vec::new();
    packet.encode_body(&mut body).unwrap();
    assert_eq!(body, [0xaa, 0, 0, 0, 1]);

    assert_eq!(
        packet.encode_packet(&mut encoder).unwrap(),
        [6, 2, 0xaa, 0, 0, 0, 1]
    );
}
