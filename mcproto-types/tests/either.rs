//! Integration tests for the protocol `Either X or Y` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{Either, TypeCodec, UnsignedByte, VarInt};

#[test]
fn x_branch_has_a_true_marker_and_roundtrips() {
    let value = Either::<UnsignedByte, VarInt>::X(UnsignedByte(0xab));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x01, 0xab]);

    let mut input = encoded.as_slice();
    assert_eq!(Either::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn y_branch_has_a_false_marker_and_roundtrips() {
    let value = Either::<UnsignedByte, VarInt>::Y(VarInt(25565));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00, 0xdd, 0xc7, 0x01]);

    let mut input = encoded.as_slice();
    assert_eq!(Either::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn invalid_branch_marker_is_rejected() {
    let error = Either::<UnsignedByte, VarInt>::decode(&mut [0x02].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidBooleanValue { value: 2 })
    );
    assert_eq!(error.codec(), CodecKind::Boolean);
    assert_eq!(error.contexts(), &[CodecKind::Either]);
    assert_eq!(error.operation(), CodecOperation::Read);
}

#[test]
fn selected_branch_requires_its_complete_payload() {
    let x_error = Either::<UnsignedByte, VarInt>::decode(&mut [0x01].as_slice()).unwrap_err();
    assert_eq!(x_error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(x_error.codec(), CodecKind::UnsignedByte);
    assert_eq!(x_error.contexts(), &[CodecKind::Either]);

    let y_error = Either::<UnsignedByte, VarInt>::decode(&mut [0x00, 0x80].as_slice()).unwrap_err();
    assert_eq!(y_error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(y_error.codec(), CodecKind::VarInt);
    assert_eq!(y_error.contexts(), &[CodecKind::Either]);
}

#[test]
fn helper_methods_preserve_the_selected_branch() {
    let x = Either::<UnsignedByte, VarInt>::X(UnsignedByte(7));
    assert!(x.is_x());
    assert!(!x.is_y());
    assert_eq!(x.as_ref(), Either::X(&UnsignedByte(7)));
    assert_eq!(x.into_x(), Some(UnsignedByte(7)));

    let y = Either::<UnsignedByte, VarInt>::Y(VarInt(9));
    assert!(y.is_y());
    assert_eq!(y.into_y(), Some(VarInt(9)));
}

#[test]
fn decoder_leaves_trailing_bytes_for_the_next_field() {
    let mut input = [0x01, 0xab, 0xcd].as_slice();
    assert_eq!(
        Either::<UnsignedByte, VarInt>::decode(&mut input).unwrap(),
        Either::X(UnsignedByte(0xab))
    );
    assert_eq!(input, [0xcd]);
}
