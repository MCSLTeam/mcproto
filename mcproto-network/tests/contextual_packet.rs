//! Tests for context-aware packet fields.

use mcproto_network::{DecodePacket, EncodePacket, PacketCodec};
use mcproto_types::{Boolean, Optional, UnsignedByte};

#[derive(Debug, PartialEq, PacketCodec)]
#[packet(
    name = "contextual_packet",
    id = 0x2a,
    state = Configuration,
    direction = Serverbound,
)]
struct ContextualPacket {
    has_value: Boolean,
    #[context(presence = has_value)]
    value: Optional<UnsignedByte>,
}

#[derive(Debug, PartialEq, PacketCodec)]
#[packet(
    name = "contextual_packet_clientbound",
    id = 0x2b,
    state = Configuration,
    direction = Clientbound,
)]
struct ContextualPacketClientbound {
    has_value: Boolean,
    #[context(presence = has_value)]
    value: Optional<UnsignedByte>,
}

#[test]
fn packet_codec_uses_the_preceding_field_as_optional_context() {
    let packet = ContextualPacket {
        has_value: Boolean(true),
        value: Optional::some(UnsignedByte(0xab)),
    };
    let mut body = Vec::new();

    packet.encode_body(&mut body).unwrap();

    assert_eq!(body, [0x01, 0xab]);
}

#[test]
fn packet_codec_decodes_absent_optional_fields() {
    let mut body = [0x00, 0xaa].as_slice();

    let packet = ContextualPacketClientbound::decode_body(&mut body).unwrap();

    assert_eq!(
        packet,
        ContextualPacketClientbound {
            has_value: Boolean(false),
            value: Optional::none(),
        }
    );
    assert_eq!(body, [0xaa]);
}
