//! Integration tests for the NBT wrapper type.

use std::collections::HashMap;

use fastnbt::Value;
use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{TypeCodec, nbt::Nbt};

#[test]
fn compound_roundtrips() {
    let value = Value::Compound(HashMap::from([
        (
            "name".to_owned(),
            Value::String("minecraft:stone".to_owned()),
        ),
        ("count".to_owned(), Value::Byte(4)),
    ]));

    let mut encoded = Vec::new();
    Nbt(value.clone()).encode(&mut encoded).unwrap();

    let mut input = encoded.as_slice();
    let decoded = Nbt::decode(&mut input).unwrap();
    assert_eq!(decoded, Nbt(value));
    assert!(input.is_empty());
}

#[test]
fn decoding_consumes_only_the_nbt_bytes() {
    let value = Value::Compound(HashMap::from([("count".to_owned(), Value::Int(7))]));

    let mut encoded = Vec::new();
    Nbt(value).encode(&mut encoded).unwrap();
    encoded.extend_from_slice(&[0xaa, 0xbb]);

    let mut input = encoded.as_slice();
    let _ = Nbt::decode(&mut input).unwrap();
    assert_eq!(input, [0xaa, 0xbb]);
}

#[test]
fn empty_input_is_invalid_nbt() {
    let mut input = [].as_slice();
    let error = Nbt::decode(&mut input).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.codec(), CodecKind::Nbt);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 0);
}
