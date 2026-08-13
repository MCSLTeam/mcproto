//! Integration tests for the protocol `Uuid` type.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation};
use mcproto_types::{TypeCodec, basic::Uuid};

const BYTES: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
];

#[test]
fn encodes_and_decodes_bytes_in_network_order() {
    let value = Uuid::from_bytes(BYTES);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, BYTES);

    let mut input = encoded.as_slice();
    assert_eq!(Uuid::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn decoding_consumes_only_the_uuid_bytes() {
    let value = Uuid::from_bytes(BYTES);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    encoded.extend_from_slice(&[0xaa, 0xbb]);

    let mut input = encoded.as_slice();
    let _ = Uuid::decode(&mut input).unwrap();
    assert_eq!(input, [0xaa, 0xbb]);
}

#[test]
fn from_bytes_and_into_bytes_roundtrip() {
    let value = Uuid::from_bytes(BYTES);
    assert_eq!(value.into_bytes(), BYTES);
}

#[test]
fn truncated_input_reports_eof_with_uuid_codec() {
    let mut input = [0x00; 15].as_slice();
    let error = Uuid::decode(&mut input).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::Uuid);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 15);
}
