//! Integration tests for `ProtocolEnum` derive.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    ProtocolEnum, TypeCodec,
    basic::{Byte, VarInt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
enum GameMode {
    Survival = 0,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = Byte)]
enum TooWideForByte {
    Value = 128,
}

#[test]
fn derived_varint_enum_roundtrips() {
    let mut encoded = Vec::new();
    GameMode::Adventure.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x02]);

    let mut input = encoded.as_slice();
    assert_eq!(GameMode::decode(&mut input).unwrap(), GameMode::Adventure);
    assert!(input.is_empty());
}

#[test]
fn auto_assigned_discriminants_are_encoded() {
    let mut encoded = Vec::new();
    GameMode::Spectator.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x03]);
}

#[test]
fn unknown_value_is_rejected_after_consuming_the_representation() {
    let mut input = [0x04].as_slice();
    let error = GameMode::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 4 })
    );
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}

#[test]
fn out_of_range_discriminant_is_rejected_before_writing() {
    let mut encoded = Vec::new();
    let error = TooWideForByte::Value.encode(&mut encoded).unwrap_err();

    assert!(encoded.is_empty());
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::EnumDiscriminantOutOfRange {
            value: 128,
        })
    );
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn malformed_representation_preserves_its_codec_error() {
    let mut input = [0x80].as_slice();
    let error = GameMode::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.contexts(), &[CodecKind::Enum]);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}
