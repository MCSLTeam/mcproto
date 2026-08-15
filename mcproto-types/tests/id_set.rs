//! Integration tests for the protocol `ID Set` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{TypeCodec, basic::Identifier, contextual::IdSet};

#[test]
fn tag_reference_uses_zero_type_and_identifier() {
    let tag = Identifier::new("minecraft:logs").unwrap();
    let value = IdSet::tag(tag.clone());

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded[0], 0x00);

    let mut input = encoded.as_slice();
    assert_eq!(IdSet::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
    assert_eq!(value.tag_name(), Some(&tag));
}

#[test]
fn empty_inline_set_uses_type_one() {
    let value = IdSet::inline(Vec::new());
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    assert_eq!(encoded, [0x01]);
    let mut input = encoded.as_slice();
    assert_eq!(IdSet::decode(&mut input).unwrap(), value);
}

#[test]
fn inline_ids_use_length_plus_one_and_roundtrip() {
    let value = IdSet::inline(vec![0, 127, 25565]);
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    assert_eq!(encoded, [0x04, 0x00, 0x7f, 0xdd, 0xc7, 0x01]);
    let mut input = encoded.as_slice();
    assert_eq!(IdSet::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn decoding_preserves_trailing_bytes() {
    let mut input = [0x02, 0x07, 0xff].as_slice();
    assert_eq!(IdSet::decode(&mut input).unwrap(), IdSet::inline(vec![7]));
    assert_eq!(input, [0xff]);
}

#[test]
fn negative_type_is_rejected() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x0f].as_slice();
    let error = IdSet::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidIdSetType { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::IdSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn negative_inline_id_is_rejected_before_writing() {
    let mut encoded = Vec::new();
    let error = IdSet::inline(vec![0, -1]).encode(&mut encoded).unwrap_err();

    assert!(encoded.is_empty());
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidRegistryId {
            value: -1,
            max: i32::MAX,
        })
    );
    assert_eq!(error.codec(), CodecKind::IdSet);
    assert_eq!(error.operation(), CodecOperation::Write);
}

#[test]
fn negative_decoded_id_reports_total_progress() {
    let mut input = [0x02, 0xff, 0xff, 0xff, 0xff, 0x0f].as_slice();
    let error = IdSet::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidRegistryId {
            value: -1,
            max: i32::MAX,
        })
    );
    assert_eq!(error.codec(), CodecKind::IdSet);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 6);
}

#[test]
fn truncated_tag_name_keeps_id_set_context() {
    let mut input = [0x00].as_slice();
    let error = IdSet::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.context(), Some(CodecKind::IdSet));
    assert_eq!(error.operation(), CodecOperation::Read);
}

#[test]
fn malformed_inline_id_keeps_varint_error() {
    let mut input = [0x02, 0xff, 0xff, 0xff, 0xff, 0x80].as_slice();
    let error = IdSet::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooLong { max_bytes: 5 })
    );
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.contexts(), &[CodecKind::IdSet]);
}

#[test]
fn accessors_and_conversions_preserve_variants() {
    let tag: IdSet = Identifier::new("minecraft:logs").unwrap().into();
    assert!(tag.is_tag());
    assert!(!tag.is_inline());

    let inline: IdSet = vec![1, 2].into();
    assert!(inline.is_inline());
    assert_eq!(inline.ids(), Some([1, 2].as_slice()));
    assert_eq!(inline.into_ids(), Some(vec![1, 2]));
}
