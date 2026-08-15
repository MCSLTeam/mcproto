//! Integration tests for the protocol `ID or X` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    TypeCodec,
    basic::{UnsignedByte, VarInt},
    contextual::IdOr,
};

#[test]
fn inline_value_uses_zero_selector_and_payload() {
    let value = IdOr::inline(UnsignedByte(0xab));

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00, 0xab]);

    let mut input = [0x00, 0xab, 0xcd].as_slice();
    assert_eq!(IdOr::<UnsignedByte>::decode(&mut input).unwrap(), value);
    assert_eq!(input, [0xcd]);
}

#[test]
fn registry_id_is_incremented_on_the_wire() {
    let cases = [
        (IdOr::<UnsignedByte>::id(0), vec![0x01]),
        (IdOr::<UnsignedByte>::id(127), vec![0x80, 0x01]),
    ];

    for (value, expected) in cases {
        let mut encoded = Vec::new();
        value.encode(&mut encoded).unwrap();
        assert_eq!(encoded, expected);

        let mut input = encoded.as_slice();
        assert_eq!(IdOr::<UnsignedByte>::decode(&mut input).unwrap(), value);
        assert!(input.is_empty());
    }
}

#[test]
fn inline_value_supports_variable_size_type_codecs() {
    let value = IdOr::inline(VarInt(25565));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    assert_eq!(encoded, [0x00, 0xdd, 0xc7, 0x01]);
}

#[test]
fn negative_selector_is_rejected() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x0f].as_slice();
    let error = IdOr::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidIdOrSelector { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::IdOr);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn invalid_registry_ids_are_rejected_before_writing() {
    for id in [-1, i32::MAX] {
        let mut encoded = Vec::new();
        let error = IdOr::<UnsignedByte>::id(id)
            .encode(&mut encoded)
            .unwrap_err();

        assert!(encoded.is_empty());
        assert_eq!(
            error.kind(),
            CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidRegistryId {
                value: id,
                max: i32::MAX - 1,
            })
        );
        assert_eq!(error.codec(), CodecKind::IdOr);
        assert_eq!(error.operation(), CodecOperation::Write);
        assert_eq!(error.bytes_processed(), 0);
    }
}

#[test]
fn truncated_inline_payload_keeps_its_codec_error() {
    let mut input = [0x00].as_slice();
    let error = IdOr::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::UnsignedByte);
    assert_eq!(error.contexts(), &[CodecKind::IdOr]);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn malformed_selector_keeps_the_varint_error() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x80].as_slice();
    let error = IdOr::<UnsignedByte>::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooLong { max_bytes: 5 })
    );
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.contexts(), &[CodecKind::IdOr]);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn accessors_preserve_variant_information() {
    let reference = IdOr::<UnsignedByte>::id(4);
    assert!(reference.is_id());
    assert!(!reference.is_inline());
    assert_eq!(reference.registry_id(), Some(4));
    assert_eq!(reference.inline_value(), None);

    let inline: IdOr<_> = UnsignedByte(7).into();
    assert!(inline.is_inline());
    assert_eq!(inline.inline_value(), Some(&UnsignedByte(7)));
    assert_eq!(inline.as_ref(), IdOr::Inline(&UnsignedByte(7)));
    assert_eq!(inline.into_inline(), Some(UnsignedByte(7)));
}
