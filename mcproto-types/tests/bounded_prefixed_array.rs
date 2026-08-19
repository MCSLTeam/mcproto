//! Tests for field-specific protocol array limits.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{BoundedPrefixedArray, TypeCodec, UnsignedByte};

#[test]
fn bounded_array_roundtrips_at_its_limit() {
    let value = BoundedPrefixedArray::<UnsignedByte, 3>::new(vec![
        UnsignedByte(1),
        UnsignedByte(2),
        UnsignedByte(3),
    ])
    .unwrap();
    let mut encoded = Vec::new();

    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x03, 0x01, 0x02, 0x03]);

    let mut input = encoded.as_slice();
    assert_eq!(
        BoundedPrefixedArray::<UnsignedByte, 3>::decode(&mut input).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn construction_rejects_too_many_elements() {
    let error = BoundedPrefixedArray::<UnsignedByte, 3>::new(vec![
        UnsignedByte(1),
        UnsignedByte(2),
        UnsignedByte(3),
        UnsignedByte(4),
    ])
    .unwrap_err();

    assert_eq!(error.max_length, 3);
    assert_eq!(error.actual_length, 4);
}

#[test]
fn decoding_rejects_an_overlong_prefix_before_reading_elements() {
    let mut input = [0x04, 0x01, 0x02, 0x03, 0x04].as_slice();
    let error = BoundedPrefixedArray::<UnsignedByte, 3>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::LengthOutOfRange {
            max: 3,
            actual: 4,
        })
    );
    assert_eq!(error.codec(), CodecKind::PrefixedArray);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
    assert_eq!(input, [0x01, 0x02, 0x03, 0x04]);
}

#[test]
fn empty_array_is_always_valid() {
    let value = BoundedPrefixedArray::<UnsignedByte, 0>::default();
    assert!(value.is_empty());

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);
}
