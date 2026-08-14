//! Integration tests for the contextual `Array of X` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    ContextualCodec,
    basic::{UnsignedByte, VarInt},
    contextual::{Array, Context, Optional},
};

#[test]
fn zero_length_array_has_no_wire_bytes() {
    let values = Array::<UnsignedByte>::default();
    let context = Context::for_array_length(0);

    let mut encoded = Vec::new();
    values.encode_with_context(&mut encoded, &context).unwrap();
    assert!(encoded.is_empty());

    let mut input = [].as_slice();
    assert_eq!(
        Array::<UnsignedByte>::decode_with_context(&mut input, &context).unwrap(),
        values
    );
}

#[test]
fn exact_length_array_roundtrips_and_preserves_trailing_bytes() {
    let values = Array(vec![UnsignedByte(1), UnsignedByte(2), UnsignedByte(3)]);
    let context = Context::for_array_length(3);
    let mut encoded = Vec::new();
    values.encode_with_context(&mut encoded, &context).unwrap();
    assert_eq!(encoded, [1, 2, 3]);

    let mut input = [1, 2, 3, 4].as_slice();
    assert_eq!(
        Array::<UnsignedByte>::decode_with_context(&mut input, &context).unwrap(),
        values
    );
    assert_eq!(input, [4]);
}

#[test]
fn generic_array_supports_variable_size_type_codecs() {
    let values = Array(vec![VarInt(1), VarInt(25565)]);
    let context = Context::for_array_length(2);
    let mut encoded = Vec::new();
    values.encode_with_context(&mut encoded, &context).unwrap();
    assert_eq!(encoded, [0x01, 0xdd, 0xc7, 0x01]);
}

#[test]
fn encoding_rejects_a_length_mismatch_before_writing() {
    let values = Array(vec![UnsignedByte(1)]);
    let mut encoded = Vec::new();
    let error = values
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
    assert_eq!(error.codec(), CodecKind::Array);
    assert_eq!(error.operation(), CodecOperation::Write);
}

#[test]
fn missing_length_context_is_rejected() {
    let values = Array(vec![UnsignedByte(1)]);
    let error = values
        .encode_with_context(&mut Vec::new(), &Context::present())
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::MissingContext {
            required: mcproto_codec::error::ContextRequirement::Length,
        })
    );
    assert_eq!(error.codec(), CodecKind::Array);
}

#[test]
fn truncated_array_reports_the_inner_codec_error() {
    let mut input = [0x01].as_slice();
    let error =
        Array::<UnsignedByte>::decode_with_context(&mut input, &Context::for_array_length(2))
            .unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::UnsignedByte);
    assert_eq!(error.contexts(), &[CodecKind::Array]);
    assert_eq!(error.operation(), CodecOperation::Read);
}

#[test]
fn contextual_elements_use_their_child_contexts() {
    let values = Array(vec![Optional::some(UnsignedByte(1)), Optional::none()]);
    let context =
        Context::for_array_length(2).with_element_contexts([Context::present(), Context::absent()]);

    let mut encoded = Vec::new();
    values.encode_with_context(&mut encoded, &context).unwrap();
    assert_eq!(encoded, [1]);

    let mut input = encoded.as_slice();
    assert_eq!(
        Array::<Optional<UnsignedByte>>::decode_with_context(&mut input, &context).unwrap(),
        values
    );
    assert!(input.is_empty());
}

#[test]
fn missing_child_context_is_rejected() {
    let values = Array(vec![Optional::some(UnsignedByte(1))]);
    let context = Context::for_array_length(1).with_element_contexts([]);
    let error = values
        .encode_with_context(&mut Vec::new(), &context)
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::MissingContext {
            required: mcproto_codec::error::ContextRequirement::ElementContext,
        })
    );
    assert_eq!(error.codec(), CodecKind::Array);
}
