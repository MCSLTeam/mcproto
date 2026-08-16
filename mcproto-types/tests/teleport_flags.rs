//! Integration tests for the protocol `Teleport Flags` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation};
use mcproto_types::{TeleportFlags, TypeCodec};

#[test]
fn constants_match_the_protocol_masks() {
    let masks = [
        (TeleportFlags::RELATIVE_X, 0x0001),
        (TeleportFlags::RELATIVE_Y, 0x0002),
        (TeleportFlags::RELATIVE_Z, 0x0004),
        (TeleportFlags::RELATIVE_YAW, 0x0008),
        (TeleportFlags::RELATIVE_PITCH, 0x0010),
        (TeleportFlags::RELATIVE_VELOCITY_X, 0x0020),
        (TeleportFlags::RELATIVE_VELOCITY_Y, 0x0040),
        (TeleportFlags::RELATIVE_VELOCITY_Z, 0x0080),
        (TeleportFlags::ROTATE_VELOCITY, 0x0100),
    ];

    for (flag, mask) in masks {
        assert_eq!(flag.bits(), mask);
    }
}

#[test]
fn all_protocol_flags_use_a_big_endian_int() {
    let flags = TeleportFlags::all();
    let mut encoded = Vec::new();
    flags.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00, 0x00, 0x01, 0xff]);

    let mut input = encoded.as_slice();
    assert_eq!(TeleportFlags::decode(&mut input).unwrap(), flags);
    assert!(input.is_empty());
}

#[test]
fn flag_operations_preserve_independent_axes() {
    let flags = TeleportFlags::RELATIVE_X
        | TeleportFlags::RELATIVE_PITCH
        | TeleportFlags::RELATIVE_VELOCITY_Z;

    assert!(flags.contains(TeleportFlags::RELATIVE_X));
    assert!(flags.contains(TeleportFlags::RELATIVE_PITCH));
    assert!(flags.contains(TeleportFlags::RELATIVE_VELOCITY_Z));
    assert!(!flags.intersects(TeleportFlags::RELATIVE_Y | TeleportFlags::RELATIVE_Z));
}

#[test]
fn decoding_retains_unknown_bits_and_trailing_bytes() {
    let mut input = [0x80, 0x00, 0x02, 0x00, 0xaa].as_slice();
    let flags = TeleportFlags::decode(&mut input).unwrap();

    assert_eq!(flags.bits(), 0x8000_0200);
    assert_eq!(flags & TeleportFlags::all(), TeleportFlags::empty());
    assert_eq!(input, [0xaa]);

    let mut encoded = Vec::new();
    flags.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x80, 0x00, 0x02, 0x00]);
}

#[test]
fn truncated_input_keeps_int_and_teleport_context() {
    let mut input = [0x00, 0x00, 0x01].as_slice();
    let error = TeleportFlags::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Int);
    assert_eq!(error.contexts(), &[CodecKind::TeleportFlags]);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 3);
}
