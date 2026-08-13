//! Integration tests for contextual protocol codecs.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    ContextualCodec,
    basic::{UnsignedByte, VarInt},
    contextual::{Context, Optional},
};

#[test]
fn context_constructors_express_presence() {
    assert_eq!(Context::new(true), Context::PRESENT);
    assert_eq!(Context::present(), Context::PRESENT);
    assert!(Context::PRESENT.is_present());

    assert_eq!(Context::new(false), Context::ABSENT);
    assert_eq!(Context::absent(), Context::ABSENT);
    assert!(!Context::ABSENT.is_present());
}

#[test]
fn present_context_encodes_and_decodes_the_value() {
    let value = Optional::some(UnsignedByte(0xab));
    let mut encoded = Vec::new();
    value
        .encode_with_context(&mut encoded, &Context::present())
        .unwrap();
    assert_eq!(encoded, [0xab]);

    let mut input = encoded.as_slice();
    assert_eq!(
        Optional::<UnsignedByte>::decode_with_context(&mut input, &Context::present()).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn absent_context_consumes_and_produces_no_bytes() {
    let value = Optional::<UnsignedByte>::none();
    let mut encoded = Vec::new();
    value
        .encode_with_context(&mut encoded, &Context::absent())
        .unwrap();
    assert!(encoded.is_empty());

    let mut input = [0xab].as_slice();
    assert_eq!(
        Optional::<UnsignedByte>::decode_with_context(&mut input, &Context::absent()).unwrap(),
        value
    );
    assert_eq!(input, [0xab]);
}

#[test]
fn generic_implementation_supports_any_type_codec() {
    let value = Optional::some(VarInt(25565));
    let mut encoded = Vec::new();
    value
        .encode_with_context(&mut encoded, &Context::present())
        .unwrap();
    assert_eq!(encoded, [0xdd, 0xc7, 0x01]);
}

#[test]
fn encoding_rejects_value_context_mismatches() {
    let cases = [
        (
            Optional::<UnsignedByte>::none(),
            Context::present(),
            InvalidEncodingReason::OptionalValueMismatch {
                context_present: true,
                value_present: false,
            },
        ),
        (
            Optional::some(UnsignedByte(1)),
            Context::absent(),
            InvalidEncodingReason::OptionalValueMismatch {
                context_present: false,
                value_present: true,
            },
        ),
    ];

    for (value, context, reason) in cases {
        let error = value
            .encode_with_context(&mut Vec::new(), &context)
            .unwrap_err();
        assert_eq!(error.kind(), CodecErrorKind::InvalidEncoding(reason));
        assert_eq!(error.codec(), CodecKind::Optional);
        assert_eq!(error.operation(), CodecOperation::Write);
        assert_eq!(error.bytes_processed(), 0);
    }
}

#[test]
fn inner_errors_keep_their_codec_and_optional_context() {
    let mut input = [0x02].as_slice();
    let error = Optional::<mcproto_types::basic::Boolean>::decode_with_context(
        &mut input,
        &Context::present(),
    )
    .unwrap_err();

    assert_eq!(error.codec(), CodecKind::Boolean);
    assert_eq!(error.contexts(), &[CodecKind::Optional]);
    assert_eq!(error.operation(), CodecOperation::Read);
}
