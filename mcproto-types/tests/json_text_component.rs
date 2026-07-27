use std::{
    error::Error as _,
    io::{self, Read, Write},
};

use mcproto_codec::{
    error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason},
    varint::VarIntWrite,
};
use mcproto_types::{
    TypeCodec,
    json_text_component::{JsonTextComponent, JsonValue},
};
use serde_json::json;

fn roundtrip(value: JsonTextComponent) -> Vec<u8> {
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    let mut input = encoded.as_slice();
    assert_eq!(JsonTextComponent::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
    encoded
}

fn encode_raw_json(json: &str) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.write_varint(json.len() as i32).unwrap();
    encoded.extend_from_slice(json.as_bytes());
    encoded
}

#[test]
fn plain_text_is_a_json_string() {
    assert_eq!(
        roundtrip(JsonTextComponent::text("Hello")),
        [7, b'"', b'H', b'e', b'l', b'l', b'o', b'"']
    );
}

#[test]
fn object_component_roundtrips() {
    roundtrip(JsonTextComponent(json!({
        "text": "Hello",
        "bold": true,
        "color": "gold"
    })));
}

#[test]
fn array_component_roundtrips() {
    roundtrip(JsonTextComponent(json!([
        { "text": "Hello" },
        { "text": " world", "italic": true }
    ])));
}

#[test]
fn chinese_uses_standard_utf8() {
    let encoded = roundtrip(JsonTextComponent::text("中"));
    assert_eq!(encoded, [5, b'"', 0xe4, 0xb8, 0xad, b'"']);
}

#[test]
fn emoji_uses_four_byte_standard_utf8() {
    let encoded = roundtrip(JsonTextComponent::text("😀"));
    assert_eq!(encoded, [6, b'"', 0xf0, 0x9f, 0x98, 0x80, b'"']);
}

#[test]
fn json_escaping_is_counted_as_wire_text() {
    assert_eq!(
        roundtrip(JsonTextComponent::text("\n")),
        [4, b'"', b'\\', b'n', b'"']
    );
}

#[test]
fn decoder_accepts_values_above_the_vanilla_encoding_limit() {
    let text = "a".repeat(JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS + 1);
    let json = format!("\"{text}\"");
    let encoded = encode_raw_json(&json);

    assert_eq!(
        JsonTextComponent::decode(&mut encoded.as_slice()).unwrap(),
        JsonTextComponent::text(text)
    );
}

#[test]
fn decoder_accepts_the_document_length_boundary() {
    let text = "a".repeat(JsonTextComponent::MAX_DECODE_UTF16_CODE_UNITS - 2);
    let json = format!("\"{text}\"");
    assert_eq!(
        json.encode_utf16().count(),
        JsonTextComponent::MAX_DECODE_UTF16_CODE_UNITS
    );
    let encoded = encode_raw_json(&json);

    assert_eq!(
        JsonTextComponent::decode(&mut encoded.as_slice()).unwrap(),
        JsonTextComponent::text(text)
    );
}

#[test]
fn decoder_rejects_too_many_utf16_code_units() {
    let text = "a".repeat(JsonTextComponent::MAX_DECODE_UTF16_CODE_UNITS - 1);
    let json = format!("\"{text}\"");
    let encoded = encode_raw_json(&json);
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooManyUtf16CodeUnits {
            max_code_units: JsonTextComponent::MAX_DECODE_UTF16_CODE_UNITS,
        })
    );
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn decoder_rejects_declared_byte_length_above_limit_before_allocating() {
    let mut encoded = Vec::new();
    encoded
        .write_varint((JsonTextComponent::MAX_DECODE_BYTES + 1) as i32)
        .unwrap();
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: JsonTextComponent::MAX_DECODE_BYTES,
        })
    );
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn encoder_accepts_the_vanilla_document_length_boundary() {
    let text = "a".repeat(JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS - 2);
    let value = JsonTextComponent::text(text);
    let encoded = roundtrip(value);
    assert_eq!(
        encoded.len(),
        JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS + 3
    );
    assert!(encoded.len() <= JsonTextComponent::MAX_ENCODE_ENCODED_BYTES);
}

#[test]
fn encoder_rejects_above_the_vanilla_utf16_limit_before_writing() {
    let text = "a".repeat(JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS - 1);
    let mut output = Vec::new();
    let error = JsonTextComponent::text(text)
        .encode(&mut output)
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooManyUtf16CodeUnits {
            max_code_units: JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS,
        })
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
    assert!(output.is_empty());
}

#[test]
fn encoder_rejects_above_the_vanilla_byte_limit_before_writing() {
    let text = "中".repeat(JsonTextComponent::MAX_ENCODE_UTF16_CODE_UNITS);
    let mut output = Vec::new();
    let error = JsonTextComponent::text(text)
        .encode(&mut output)
        .unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: JsonTextComponent::MAX_ENCODE_BYTES,
        })
    );
    assert!(output.is_empty());
}

#[test]
fn negative_length_is_rejected() {
    let encoded = [0xff, 0xff, 0xff, 0xff, 0x0f];
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::NegativeLength { value: -1 })
    );
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn truncated_payload_is_unexpected_eof() {
    let encoded = [5, b'"', b'h'];
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::JsonTextComponent);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn truncated_prefix_preserves_varint_context() {
    let error = JsonTextComponent::decode(&mut [0x80].as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.context(), Some(CodecKind::JsonTextComponent));
    assert_eq!(error.bytes_processed(), 1);
}

#[test]
fn invalid_utf8_is_rejected() {
    let encoded = [2, 0xc3, 0x28];
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidUtf8 {
            valid_up_to: 0,
            error_len: Some(1),
        })
    );
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn invalid_json_is_rejected_with_source() {
    let encoded = encode_raw_json("{\"text\":}");
    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidJson)
    );
    assert_eq!(error.codec(), CodecKind::JsonTextComponent);
    assert_eq!(error.bytes_processed(), encoded.len());
    assert!(error.source().is_some());
}

#[test]
fn decoding_consumes_only_one_component() {
    let mut encoded = encode_raw_json("\"hi\"");
    encoded.push(0xaa);
    let mut input = encoded.as_slice();
    assert_eq!(
        JsonTextComponent::decode(&mut input).unwrap(),
        JsonTextComponent::text("hi")
    );
    assert_eq!(input, [0xaa]);
}

struct FailAfterWriter {
    remaining: usize,
}

impl Write for FailAfterWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("injected write failure"));
        }
        let written = buffer.len().min(self.remaining);
        self.remaining -= written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn partial_payload_write_reports_exact_progress() {
    let error = JsonTextComponent::text("hello")
        .encode(&mut FailAfterWriter { remaining: 4 })
        .unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::JsonTextComponent);
    assert_eq!(error.bytes_processed(), 4);
}

#[test]
fn partial_prefix_write_preserves_varint_context() {
    let value = JsonTextComponent::text("a".repeat(128));
    let error = value
        .encode(&mut FailAfterWriter { remaining: 1 })
        .unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.context(), Some(CodecKind::JsonTextComponent));
    assert_eq!(error.bytes_processed(), 1);
}

struct FailAfterReader {
    input: &'static [u8],
}

impl Read for FailAfterReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.input.is_empty() {
            return Err(io::Error::other("injected read failure"));
        }
        let read = buffer.len().min(self.input.len());
        buffer[..read].copy_from_slice(&self.input[..read]);
        self.input = &self.input[read..];
        Ok(read)
    }
}

#[test]
fn partial_payload_read_reports_exact_progress() {
    let mut reader = FailAfterReader {
        input: &[5, b'"', b'h'],
    };
    let error = JsonTextComponent::decode(&mut reader).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::JsonTextComponent);
    assert_eq!(error.bytes_processed(), 3);
}

#[test]
fn parses_json_constructor() {
    assert_eq!(
        JsonTextComponent::from_json_str("{\"text\":\"hello\"}").unwrap(),
        JsonTextComponent(JsonValue::Object(serde_json::Map::from_iter([(
            "text".to_owned(),
            JsonValue::String("hello".to_owned()),
        )])))
    );
}
