//! Integration tests for the protocol `Sound Event` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{Float, Identifier, SoundEvent, TypeCodec};

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

#[test]
fn variable_range_sound_has_false_marker_and_no_float() {
    let value = SoundEvent::variable(identifier("minecraft:entity.player.levelup"));
    let mut expected = Vec::new();
    value.sound_name.encode(&mut expected).unwrap();
    expected.push(0x00);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(SoundEvent::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
    assert!(!value.has_fixed_range());
}

#[test]
fn fixed_range_sound_has_true_marker_and_float() {
    let value = SoundEvent::fixed(identifier("minecraft:block.note_block.harp"), Float(16.0));
    let mut expected = Vec::new();
    value.sound_name.encode(&mut expected).unwrap();
    expected.extend_from_slice(&[0x01, 0x41, 0x80, 0x00, 0x00]);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(SoundEvent::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
    assert!(value.has_fixed_range());
}

#[test]
fn decoding_preserves_trailing_bytes() {
    let value = SoundEvent::variable(identifier("minecraft:ambient.cave"));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    encoded.push(0xaa);

    let mut input = encoded.as_slice();
    assert_eq!(SoundEvent::decode(&mut input).unwrap(), value);
    assert_eq!(input, [0xaa]);
}

#[test]
fn invalid_presence_marker_keeps_nested_context() {
    let sound_name = identifier("minecraft:ambient.cave");
    let mut encoded = Vec::new();
    sound_name.encode(&mut encoded).unwrap();
    encoded.push(0x02);

    let error = SoundEvent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidBooleanValue { value: 2 })
    );
    assert_eq!(error.codec(), CodecKind::Boolean);
    assert_eq!(
        error.contexts(),
        &[CodecKind::PrefixedOptional, CodecKind::SoundEvent]
    );
    assert_eq!(error.operation(), CodecOperation::Read);
}

#[test]
fn truncated_fixed_range_keeps_float_context() {
    let sound_name = identifier("minecraft:ambient.cave");
    let mut encoded = Vec::new();
    sound_name.encode(&mut encoded).unwrap();
    encoded.extend_from_slice(&[0x01, 0x41, 0x80]);

    let error = SoundEvent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Float);
    assert_eq!(
        error.contexts(),
        &[
            CodecKind::Optional,
            CodecKind::PrefixedOptional,
            CodecKind::SoundEvent,
        ]
    );
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 2);
}
