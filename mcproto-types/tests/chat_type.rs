//! Integration tests for the protocol `Chat Type` value.

use std::collections::HashMap;

use fastnbt::Value;
use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    ChatDecoration, ChatType, ChatTypeParameter, Nbt, PrefixedArray, PrefixedString, TypeCodec,
};

fn empty_style() -> Nbt {
    Nbt(Value::Compound(HashMap::new()))
}

fn decoration(translation_key: &str, parameters: Vec<ChatTypeParameter>) -> ChatDecoration {
    ChatDecoration::new(
        PrefixedString(translation_key.to_owned()),
        PrefixedArray(parameters),
        empty_style(),
    )
}

#[test]
fn parameter_values_are_varint_enums() {
    let values = [
        (ChatTypeParameter::Sender, 0x00),
        (ChatTypeParameter::Target, 0x01),
        (ChatTypeParameter::Content, 0x02),
    ];

    for (value, expected) in values {
        let mut encoded = Vec::new();
        value.encode(&mut encoded).unwrap();
        assert_eq!(encoded, [expected]);

        let mut input = encoded.as_slice();
        assert_eq!(ChatTypeParameter::decode(&mut input).unwrap(), value);
        assert!(input.is_empty());
    }
}

#[test]
fn chat_is_encoded_before_narration() {
    let value = ChatType::new(
        decoration(
            "a",
            vec![ChatTypeParameter::Sender, ChatTypeParameter::Content],
        ),
        decoration("b", vec![ChatTypeParameter::Target]),
    );

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    assert_eq!(
        encoded,
        [
            0x01, b'a', 0x02, 0x00, 0x02, 0x0a, 0x00, // Chat.
            0x01, b'b', 0x01, 0x01, 0x0a, 0x00, // Narration.
        ]
    );
}

#[test]
fn chat_type_roundtrips_and_preserves_trailing_bytes() {
    let value = ChatType::new(
        decoration(
            "chat.type.text",
            vec![ChatTypeParameter::Sender, ChatTypeParameter::Content],
        ),
        decoration(
            "chat.type.text.narrate",
            vec![ChatTypeParameter::Sender, ChatTypeParameter::Content],
        ),
    );
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    encoded.extend_from_slice(&[0xaa, 0xbb]);

    let mut input = encoded.as_slice();
    assert_eq!(ChatType::decode(&mut input).unwrap(), value);
    assert_eq!(input, [0xaa, 0xbb]);
}

#[test]
fn unknown_parameter_keeps_nested_context() {
    let mut input = [
        0x01, b'a', // Translation key.
        0x01, // One parameter.
        0x03, // Unknown parameter.
    ]
    .as_slice();

    let error = ChatType::decode(&mut input).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 3 })
    );
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(
        error.contexts(),
        &[
            CodecKind::PrefixedArray,
            CodecKind::ChatDecoration,
            CodecKind::ChatType,
        ]
    );
}

#[test]
fn style_is_required_even_when_empty() {
    let value = decoration("a", Vec::new());
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x01, b'a', 0x00, 0x0a, 0x00]);

    let mut missing_style = [0x01, b'a', 0x00].as_slice();
    let error = ChatDecoration::decode(&mut missing_style).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.codec(), CodecKind::Nbt);
    assert_eq!(error.contexts(), &[CodecKind::ChatDecoration]);
}
