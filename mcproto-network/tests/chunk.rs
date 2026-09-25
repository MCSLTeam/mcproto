//! Chunk packet protocol tests.

use mcproto_network::{
    Direction, EncodePacket, Packet, ProtocolState, packet::play::ChunkBatchReceived,
};
use mcproto_types::Float;

#[test]
fn chunk_batch_received_matches_protocol_777() {
    assert_eq!(ChunkBatchReceived::NAME.as_str(), "chunk_batch_received");
    assert_eq!(ChunkBatchReceived::ID.get(), 0x0b);
    assert_eq!(ChunkBatchReceived::STATE, ProtocolState::Play);
    assert_eq!(ChunkBatchReceived::DIRECTION, Direction::Serverbound);
}

#[test]
fn chunk_batch_received_encodes_chunks_per_tick() {
    let packet = ChunkBatchReceived {
        chunks_per_tick: Float(1.5),
    };
    let mut body = Vec::new();

    packet.encode_body(&mut body).unwrap();

    assert_eq!(body, [0x3f, 0xc0, 0x00, 0x00]);
}
