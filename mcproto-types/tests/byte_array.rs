//! Integration tests for the contextual protocol `Byte Array` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    ContextualCodec,
    contextual::{ByteArray, Context},
};

#[test]
fn zero_length_array_has_no_wire_bytes() {
    let value = ByteArray::default();
    let context = Context::for_array_length(0);

    let mut encoded = Vec::new();
    value.encode_with_context(&mut encoded, &context).unwrap();
    assert!(encoded.is_empty());

    let mut input = [].as_slice();
    assert_eq!(
        ByteArray::decode_with_context(&mut input, &context).unwrap(),
        value
    );
}

#[test]
fn byte_array_roundtrips_and_preserves_trailing_bytes() {
    let value = ByteArray(vec![0x00, 0x7f, 0x80, 0xff]);
    let context = Context::for_array_length(4);

    let mut encoded = Vec::new();
    value.encode_with_context(&mut encoded, &context).unwrap();
    assert_eq!(encoded, [0x00, 0x7f, 0x80, 0xff]);

    let mut input = [0x00, 0x7f, 0x80, 0xff, 0x12].as_slice();
    assert_eq!(
        ByteArray::decode_with_context(&mut input, &context).unwrap(),
        value
    );
    assert_eq!(input, [0x12]);
}

#[test]
fn encoding_rejects_a_length_mismatch_before_writing() {
    let value = ByteArray(vec![0x01]);
    let mut encoded = Vec::new();
    let error = value
        .encode_with_context(&mut encoded, &Context::for_array_length(2))
        .unwrap_err();

    assert!(encoded.is_empty());
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ArrayLengthMismatch {
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(error.codec(), CodecKind::ByteArray);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn missing_length_context_is_rejected() {
    let error = ByteArray(vec![0x01])
        .encode_with_context(&mut Vec::new(), &Context::present())
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::MissingContext {
            required: mcproto_codec::error::ContextRequirement::Length,
        })
    );
    assert_eq!(error.codec(), CodecKind::ByteArray);
    assert_eq!(error.operation(), CodecOperation::Write);
}

#[test]
fn truncated_data_reports_exact_progress() {
    let mut input = [0xab].as_slice();
    let error =
        ByteArray::decode_with_context(&mut input, &Context::for_array_length(2)).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::ByteArray);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}

#[test]
fn conversion_helpers_preserve_bytes() {
    let value: ByteArray = vec![0x12, 0x34].into();
    assert_eq!(value.len(), 2);
    assert_eq!(value.as_slice(), [0x12, 0x34]);
    assert!(!value.is_empty());
    assert_eq!(value.into_vec(), vec![0x12, 0x34]);
}
