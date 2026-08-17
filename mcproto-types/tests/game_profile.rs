//! Protocol tests for Game Profile.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    GameProfile, GameProfileProperties, GameProfileProperty, GameProfilePropertyName,
    GameProfilePropertySignature, GameProfilePropertyValue, GameProfileStringTooLong,
    GameProfileUsername, PrefixedOptional, TypeCodec, Uuid,
};

const UUID_BYTES: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
];

fn property() -> GameProfileProperty {
    GameProfileProperty {
        name: GameProfilePropertyName::new("textures").unwrap(),
        value: GameProfilePropertyValue::new("abc").unwrap(),
        signature: PrefixedOptional::some(GameProfilePropertySignature::new("sig").unwrap()),
    }
}

#[test]
fn profile_fields_follow_the_documented_wire_order() {
    let profile = GameProfile {
        uuid: Uuid::from_bytes(UUID_BYTES),
        username: GameProfileUsername::new("Alex").unwrap(),
        properties: GameProfileProperties::new(vec![property()]).unwrap(),
    };

    let mut encoded = Vec::new();
    profile.encode(&mut encoded).unwrap();

    let mut expected = UUID_BYTES.to_vec();
    expected.extend_from_slice(&[
        4, b'A', b'l', b'e', b'x', // Username
        1,    // Property count
        8, b't', b'e', b'x', b't', b'u', b'r', b'e', b's', // Name
        3, b'a', b'b', b'c', // Value
        1, 3, b's', b'i', b'g', // Signature
    ]);
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(GameProfile::decode(&mut input).unwrap(), profile);
    assert!(input.is_empty());
}

#[test]
fn empty_property_list_is_a_zero_prefix() {
    let profile = GameProfile::new(
        Uuid::from_bytes(UUID_BYTES),
        GameProfileUsername::new("Steve").unwrap(),
    );
    let mut encoded = Vec::new();
    profile.encode(&mut encoded).unwrap();
    assert_eq!(encoded.last(), Some(&0));
    assert_eq!(
        GameProfile::decode(&mut encoded.as_slice()).unwrap(),
        profile
    );
}

#[test]
fn string_limits_count_utf16_code_units() {
    let accepted = GameProfileUsername::new("😀".repeat(8)).unwrap();
    assert_eq!(accepted.as_str().encode_utf16().count(), 16);

    let error = GameProfileUsername::new("😀".repeat(9)).unwrap_err();
    assert_eq!(
        error,
        GameProfileStringTooLong {
            max_code_units: 16,
            actual_code_units: 18,
        }
    );
}

#[test]
fn decoding_rejects_an_overlong_username() {
    let mut encoded = UUID_BYTES.to_vec();
    encoded.extend_from_slice(&[17]);
    encoded.extend_from_slice(b"seventeen-letters!");

    let error = GameProfile::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooManyUtf16CodeUnits {
            max_code_units: 16,
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.contexts(), &[CodecKind::GameProfile]);
}

#[test]
fn property_count_is_limited_to_sixteen() {
    let properties = vec![property(); 16];
    assert!(GameProfileProperties::new(properties).is_ok());

    let error = GameProfileProperties::new(vec![property(); 17]).unwrap_err();
    assert_eq!(error.actual, 17);

    let decode_error = GameProfileProperties::decode(&mut [17].as_slice()).unwrap_err();
    assert_eq!(
        decode_error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::LengthOutOfRange {
            max: 16,
            actual: 17,
        })
    );
    assert_eq!(decode_error.codec(), CodecKind::PrefixedArray);
}

#[test]
fn constructors_and_conversion_helpers_preserve_values() {
    let username = GameProfileUsername::try_from("Player").unwrap();
    assert_eq!(username.as_ref(), "Player");
    assert_eq!(username.to_string(), "Player");

    let mut properties = GameProfileProperties::default();
    properties.push(property()).unwrap();
    assert_eq!(properties.len(), 1);
    assert!(!properties.is_empty());
    assert_eq!(properties.as_slice()[0].name.as_str(), "textures");
}
