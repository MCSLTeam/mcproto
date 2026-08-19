//! Handshaking packet protocol tests.

use mcproto_network::{
    EncodePacket, Packet,
    packet::handshaking::{Handshake, Intent, ServerAddress},
};
use mcproto_types::{UnsignedShort, VarInt};

#[test]
fn handshake_matches_protocol_field_order() {
    let packet = Handshake {
        protocol_version: VarInt(776),
        server_address: ServerAddress::new("localhost").unwrap(),
        server_port: UnsignedShort(25565),
        intent: Intent::Status,
    };
    let mut body = Vec::new();

    packet.encode_body(&mut body).unwrap();

    assert_eq!(Handshake::ID.get(), 0x00);
    assert_eq!(
        body,
        [
            0x88, 0x06, 0x09, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't', 0x63, 0xdd,
            0x01,
        ]
    );
}

#[test]
fn handshake_rejects_server_addresses_over_255_code_units() {
    assert!(ServerAddress::new("a".repeat(256)).is_err());
}
