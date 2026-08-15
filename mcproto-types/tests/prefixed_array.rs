//! Integration tests for the protocol `Prefixed Array of X` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    TypeCodec,
    basic::{UnsignedByte, VarInt},
    contextual::PrefixedArray,
};

#[test]
fn empty_array_has_a_zero_varint_prefix() {
    let values = PrefixedArray::<UnsignedByte>::default();

    let mut encoded = Vec::new();
    values.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = encoded.as_slice();
    assert_eq!(
        PrefixedArray::<UnsignedByte>::decode(&mut input).unwrap(),
        values
    );
    assert!(input.is_empty());
}

#[test]
fn array_roundtrips_with_its_varint_length_prefix() {
    let values = PrefixedArray(vec![UnsignedByte(1), UnsignedByte(2), UnsignedByte(3)]);

    let mut encoded = Vec::new();
    values.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x03, 0x01, 0x02, 0x03]);

    let mut input = [0x03, 0x01, 0x02, 0x03, 0x04].as_slice();
    assert_eq!(
        PrefixedArray::<UnsignedByte>::decode(&mut input).unwrap(),
        values
    );
    assert_eq!(input, [0x04]);
}

#[test]
fn generic_array_supports_variable_size_type_codecs() {
    let values = PrefixedArray(vec![VarInt(1), VarInt(25565)]);

    let mut encoded = Vec::new();
    values.encode(&mut encoded).unwrap();

    assert_eq!(encoded, [0x02, 0x01, 0xdd, 0xc7, 0x01]);
}

#[test]
fn negative_length_is_rejected() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x0f].as_slice();
    let error = PrefixedArray::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::NegativeLength { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::PrefixedArray);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn truncated_element_keeps_its_codec_and_array_context() {
    let mut input = [0x02, 0x01].as_slice();
    let error = PrefixedArray::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::UnsignedByte);
    assert_eq!(error.contexts(), &[CodecKind::PrefixedArray]);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn malformed_length_prefix_keeps_the_varint_error() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x80].as_slice();
    let error = PrefixedArray::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooLong { max_bytes: 5 })
    );
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.contexts(), &[CodecKind::PrefixedArray]);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn conversion_helpers_preserve_the_elements() {
    let values: PrefixedArray<_> = vec![UnsignedByte(1), UnsignedByte(2)].into();
    assert_eq!(values.len(), 2);
    assert_eq!(values.as_slice(), [UnsignedByte(1), UnsignedByte(2)]);
    assert!(!values.is_empty());
    assert_eq!(values.into_vec(), vec![UnsignedByte(1), UnsignedByte(2)]);
}
