use std::{
    error::Error as _,
    io::{self, Read, Write},
};

use mcproto_codec::error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{
    TypeCodec,
    text_component::{NbtCompound, NbtValue, TextComponent},
};

fn roundtrip(value: TextComponent) -> Vec<u8> {
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    let mut input = encoded.as_slice();
    assert_eq!(TextComponent::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
    encoded
}

#[test]
fn empty_string_root() {
    assert_eq!(roundtrip(TextComponent::text("")), [8, 0, 0]);
}

#[test]
fn ascii_string_root() {
    assert_eq!(
        roundtrip(TextComponent::text("Hello, world!")),
        [
            8, 0, 13, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
        ]
    );
}

#[test]
fn string_root_uses_java_modified_utf8() {
    assert_eq!(
        roundtrip(TextComponent::text("\0😀")),
        [8, 0, 8, 0xc0, 0x80, 0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80]
    );
}

#[test]
fn styled_compound_roundtrips_through_fastnbt() {
    let component = TextComponent::compound(NbtCompound::from([
        ("text".to_owned(), NbtValue::String("Hello".to_owned())),
        ("bold".to_owned(), NbtValue::Byte(1)),
        ("color".to_owned(), NbtValue::String("gold".to_owned())),
    ]));

    let encoded = roundtrip(component);
    assert_eq!(encoded.first(), Some(&10));
    assert_eq!(encoded.last(), Some(&0));
}

#[test]
fn compound_with_extra_components_roundtrips() {
    let child = NbtCompound::from([("text".to_owned(), NbtValue::String(" world".to_owned()))]);
    let component = TextComponent::compound(NbtCompound::from([
        ("text".to_owned(), NbtValue::String("Hello".to_owned())),
        (
            "extra".to_owned(),
            NbtValue::List(vec![NbtValue::Compound(child)]),
        ),
    ]));

    roundtrip(component);
}

#[test]
fn decoding_consumes_only_one_string_component() {
    let mut input = [8, 0, 2, b'h', b'i', 0xaa].as_slice();
    assert_eq!(
        TextComponent::decode(&mut input).unwrap(),
        TextComponent::text("hi")
    );
    assert_eq!(input, [0xaa]);
}

#[test]
fn decoding_consumes_only_one_compound_component() {
    let mut input = [10, 0, 0xaa].as_slice();
    assert_eq!(
        TextComponent::decode(&mut input).unwrap(),
        TextComponent::compound(NbtCompound::new())
    );
    assert_eq!(input, [0xaa]);
}

#[test]
fn invalid_root_tag_is_rejected() {
    let error = TextComponent::decode(&mut [1].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidTextComponentRootTag {
            tag: 1
        })
    );
    assert_eq!(error.codec(), CodecKind::TextComponent);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}

#[test]
fn empty_input_is_unexpected_eof() {
    let error = TextComponent::decode(&mut [].as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::TextComponent);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn truncated_string_length_is_unexpected_eof() {
    let error = TextComponent::decode(&mut [8, 0].as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.bytes_processed(), 2);
}

#[test]
fn truncated_string_payload_is_unexpected_eof() {
    let error = TextComponent::decode(&mut [8, 0, 5, b'h', b'i'].as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn invalid_modified_utf8_is_rejected_with_fastnbt_source() {
    let error = TextComponent::decode(&mut [8, 0, 1, 0xff].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.bytes_processed(), 4);
    assert!(error.source().is_some());
}

#[test]
fn truncated_compound_is_unexpected_eof_with_exact_progress() {
    let encoded = [10, 8, 0, 4, b't', b'e'];
    let error = TextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn malformed_compound_is_invalid_nbt() {
    let error = TextComponent::decode(&mut [10, 99].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.bytes_processed(), 2);
    assert!(error.source().is_some());
}

#[test]
fn string_root_accepts_maximum_modified_utf8_length() {
    let value = TextComponent::text("a".repeat(u16::MAX as usize));
    let encoded = roundtrip(value);
    assert_eq!(&encoded[..3], &[8, 0xff, 0xff]);
}

#[test]
fn string_root_rejects_too_many_modified_utf8_bytes_before_writing() {
    let value = TextComponent::text("a".repeat(u16::MAX as usize + 1));
    let mut output = Vec::new();
    let error = value.encode(&mut output).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: u16::MAX as usize,
        })
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
    assert!(output.is_empty());
}

#[test]
fn compound_rejects_oversized_nested_string_before_writing() {
    let nested = NbtCompound::from([(
        "text".to_owned(),
        NbtValue::String("a".repeat(u16::MAX as usize + 1)),
    )]);
    let component = TextComponent::compound(NbtCompound::from([(
        "hoverEvent".to_owned(),
        NbtValue::Compound(nested),
    )]));
    let mut output = Vec::new();
    let error = component.encode(&mut output).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: u16::MAX as usize,
        })
    );
    assert!(output.is_empty());
}

struct FailAfterWriter {
    remaining: usize,
}

impl Write for FailAfterWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("injected write failure"));
        }
        let written = self.remaining.min(buffer.len());
        self.remaining -= written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn partial_string_write_reports_exact_progress() {
    let error = TextComponent::text("hello")
        .encode(&mut FailAfterWriter { remaining: 4 })
        .unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 4);
}

#[test]
fn partial_compound_write_reports_exact_progress() {
    let component = TextComponent::compound(NbtCompound::from([(
        "text".to_owned(),
        NbtValue::String("hello".to_owned()),
    )]));
    let error = component
        .encode(&mut FailAfterWriter { remaining: 5 })
        .unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.bytes_processed(), 5);
}

struct FailAfterReader {
    input: &'static [u8],
}

impl Read for FailAfterReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.input.is_empty() {
            return Err(io::Error::other("injected read failure"));
        }
        let read = self.input.len().min(buffer.len());
        buffer[..read].copy_from_slice(&self.input[..read]);
        self.input = &self.input[read..];
        Ok(read)
    }
}

#[test]
fn partial_compound_read_reports_exact_progress() {
    let mut reader = FailAfterReader {
        input: &[10, 8, 0, 4, b't'],
    };
    let error = TextComponent::decode(&mut reader).unwrap_err();
    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 5);
}
