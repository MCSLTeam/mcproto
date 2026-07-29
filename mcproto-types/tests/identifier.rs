//! Integration tests for Minecraft resource identifiers.

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{TypeCodec, basic::Identifier};

#[test]
fn valid_identifiers_roundtrip() {
    for value in [
        "stone",
        "minecraft:stone",
        "example_namespace:path/to.some-block",
    ] {
        let identifier = Identifier::new(value).unwrap();
        assert_eq!(identifier.as_str(), value);

        let mut encoded = Vec::new();
        identifier.encode(&mut encoded).unwrap();
        let mut input = encoded.as_slice();
        assert_eq!(Identifier::decode(&mut input).unwrap(), identifier);
        assert!(input.is_empty());
    }
}

#[test]
fn constructor_rejects_invalid_identifiers() {
    for value in [
        "",
        ":stone",
        "minecraft:",
        "MineCraft:stone",
        "a:b:c",
        "a:path\\x",
    ] {
        assert!(Identifier::new(value).is_err(), "accepted {value:?}");
    }
}

#[test]
fn decoder_rejects_invalid_identifiers_after_consuming_the_string() {
    let encoded = [0x03, b'A', b':', b'b'];
    let error = Identifier::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidIdentifier)
    );
    assert_eq!(error.codec(), CodecKind::Identifier);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}
