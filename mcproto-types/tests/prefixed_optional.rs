//! Integration tests for the protocol `Prefixed Optional X` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    TypeCodec,
    basic::{Boolean, UnsignedByte, VarInt},
    contextual::PrefixedOptional,
};

#[test]
fn present_value_has_a_true_prefix_and_roundtrips() {
    let value = PrefixedOptional::some(UnsignedByte(0xab));

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x01, 0xab]);

    let mut input = encoded.as_slice();
    assert_eq!(
        PrefixedOptional::<UnsignedByte>::decode(&mut input).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn absent_value_has_a_false_prefix_and_no_payload() {
    let value = PrefixedOptional::<UnsignedByte>::none();

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = encoded.as_slice();
    assert_eq!(
        PrefixedOptional::<UnsignedByte>::decode(&mut input).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn generic_wrapper_supports_other_type_codecs() {
    let value = PrefixedOptional::some(VarInt(25565));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x01, 0xdd, 0xc7, 0x01]);
}

#[test]
fn invalid_boolean_presence_is_rejected() {
    let mut input = [0x02].as_slice();
    let error = PrefixedOptional::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidBooleanValue { value: 2 })
    );
    assert_eq!(error.codec(), CodecKind::Boolean);
    assert_eq!(error.contexts(), &[CodecKind::PrefixedOptional]);
    assert_eq!(error.operation(), CodecOperation::Read);
}

#[test]
fn present_prefix_requires_a_complete_payload() {
    let mut input = [0x01].as_slice();
    let error = PrefixedOptional::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::UnsignedByte);
    assert_eq!(
        error.contexts(),
        &[CodecKind::Optional, CodecKind::PrefixedOptional]
    );
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn trailing_bytes_are_left_for_the_next_field() {
    let mut input = [0x01, 0xab, 0xcd].as_slice();
    assert_eq!(
        PrefixedOptional::<UnsignedByte>::decode(&mut input).unwrap(),
        PrefixedOptional::some(UnsignedByte(0xab))
    );
    assert_eq!(input, [0xcd]);
}

#[test]
fn conversion_helpers_preserve_the_optional_value() {
    let value: PrefixedOptional<_> = Some(UnsignedByte(1)).into();
    assert!(value.is_some());
    assert_eq!(value.into_option(), Some(UnsignedByte(1)));

    let absent = PrefixedOptional::<Boolean>::none();
    let option: Option<Boolean> = absent.into();
    assert_eq!(option, None);
}
