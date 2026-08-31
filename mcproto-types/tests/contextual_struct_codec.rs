//! Tests for deriving context-aware structure codecs.

use mcproto_codec::error::{CodecErrorKind, InvalidEncodingReason};
use mcproto_types::contextual::{Context, Optional};
use mcproto_types::{Boolean, ContextualCodec, ContextualStructCodec, TypeCodec, UnsignedByte};

#[derive(Debug, PartialEq, ContextualStructCodec)]
#[contextual_struct_codec(kind = TypeStruct)]
struct OptionalField {
    has_value: Boolean,
    #[context(presence = has_value)]
    value: Optional<UnsignedByte>,
}

#[test]
fn derives_presence_context_from_an_earlier_boolean_field() {
    let value = OptionalField {
        has_value: Boolean(true),
        value: Optional::some(UnsignedByte(0xab)),
    };
    let mut encoded = Vec::new();

    value
        .encode_with_context(&mut encoded, &Context::present())
        .unwrap();
    assert_eq!(encoded, [0x01, 0xab]);

    let mut input = encoded.as_slice();
    assert_eq!(
        OptionalField::decode_with_context(&mut input, &Context::present()).unwrap(),
        value
    );
    assert!(input.is_empty());
}

#[test]
fn derives_absent_optional_without_consuming_bytes() {
    let value = OptionalField {
        has_value: Boolean(false),
        value: Optional::none(),
    };
    let mut encoded = Vec::new();

    value
        .encode_with_context(&mut encoded, &Context::present())
        .unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = [0x00, 0xaa].as_slice();
    assert_eq!(
        OptionalField::decode_with_context(&mut input, &Context::present()).unwrap(),
        value
    );
    assert_eq!(input, [0xaa]);
}

#[test]
fn derives_context_mismatch_errors_instead_of_dropping_the_value() {
    let value = OptionalField {
        has_value: Boolean(true),
        value: Optional::none(),
    };
    let error = value
        .encode_with_context(&mut Vec::new(), &Context::present())
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::OptionalValueMismatch {
            context_present: true,
            value_present: false,
        })
    );
}

#[allow(dead_code)]
fn assert_type_codec_is_still_available<T: TypeCodec>() {}
