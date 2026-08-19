//! Status ping request/response packet tests.

use mcproto_network::{
    DecodePacket, Direction, EncodePacket, Packet, PacketName, ProtocolState,
    packet::status::{PingRequestStatus, PongResponseStatus},
};
use mcproto_types::Long;

#[test]
fn ping_request_encodes_timestamp_in_network_order() {
    let packet = PingRequestStatus {
        timestamp: Long(0x0102_0304_0506_0708),
    };
    let mut body = Vec::new();

    packet.encode_body(&mut body).unwrap();

    assert_eq!(body, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(PingRequestStatus::NAME, PacketName::new("ping_request"));
    assert_eq!(PingRequestStatus::ID.get(), 0x01);
    assert_eq!(PingRequestStatus::STATE, ProtocolState::Status);
    assert_eq!(PingRequestStatus::DIRECTION, Direction::Serverbound);
}

#[test]
fn pong_response_decodes_timestamp_in_network_order() {
    let mut input = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08].as_slice();

    let packet = PongResponseStatus::decode_body(&mut input).unwrap();

    assert!(input.is_empty());
    assert_eq!(packet.timestamp, Long(0x0102_0304_0506_0708));
    assert_eq!(PongResponseStatus::NAME, PacketName::new("pong_response"));
    assert_eq!(PongResponseStatus::ID.get(), 0x01);
    assert_eq!(PongResponseStatus::STATE, ProtocolState::Status);
    assert_eq!(PongResponseStatus::DIRECTION, Direction::Clientbound);
}

#[test]
fn status_ping_packets_reject_truncated_timestamps() {
    let mut pong_input = [0_u8; 7].as_slice();
    assert!(PongResponseStatus::decode_body(&mut pong_input).is_err());
}
