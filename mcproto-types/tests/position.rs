//! Integration tests for the protocol `Position` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation};
use mcproto_types::{TypeCodec, basic::Position};

#[test]
fn example_from_protocol_spec() {
    let position = Position {
        x: 18_357_644,
        y: 831,
        z: -20_882_616,
    };

    let mut encoded = Vec::new();
    position.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x46, 0x07, 0x63, 0x2c, 0x15, 0xb4, 0x83, 0x3f]);

    let mut input = encoded.as_slice();
    assert_eq!(Position::decode(&mut input).unwrap(), position);
    assert!(input.is_empty());
}

#[test]
fn packs_components_into_the_expected_bits() {
    let cases = [
        (
            Position { x: 1, y: 0, z: 0 },
            [0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00],
        ),
        (
            Position { x: 0, y: 0, z: 1 },
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00],
        ),
        (
            Position { x: 0, y: 1, z: 0 },
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
        ),
    ];

    for (position, expected) in cases {
        let mut encoded = Vec::new();
        position.encode(&mut encoded).unwrap();
        assert_eq!(encoded, expected);
    }
}

#[test]
fn component_bounds_roundtrip() {
    let positions = [
        Position {
            x: -33_554_432,
            y: -2_048,
            z: -33_554_432,
        },
        Position {
            x: 33_554_431,
            y: 2_047,
            z: 33_554_431,
        },
        Position { x: 0, y: 0, z: 0 },
    ];

    for position in positions {
        let mut encoded = Vec::new();
        position.encode(&mut encoded).unwrap();

        let mut input = encoded.as_slice();
        assert_eq!(Position::decode(&mut input).unwrap(), position);
        assert!(input.is_empty());
    }
}

#[test]
fn truncated_input_reports_eof_with_exact_progress() {
    let mut input = [0x00; 7].as_slice();
    let error = Position::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Position);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 7);
}
