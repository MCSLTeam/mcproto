use std::io::{self, Write};

use mcproto_codec::{
    error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason},
    varint::VarIntWrite,
};
use mcproto_types::{TypeCodec, basic::PrefixedString};

fn assert_roundtrip(value: &str, expected: &[u8]) {
    let value = PrefixedString(value.to_owned());
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(PrefixedString::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn empty_string() {
    assert_roundtrip("", &[0x00]);
}

#[test]
fn ascii_string() {
    assert_roundtrip(
        "Hello, world!",
        &[
            0x0d, b'H', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
        ],
    );
}

#[test]
fn chinese_character_uses_three_utf8_bytes_and_one_utf16_code_unit() {
    assert_eq!("中".encode_utf16().count(), 1);
    assert_roundtrip("中", &[0x03, 0xe4, 0xb8, 0xad]);
}

#[test]
fn emoji_uses_four_utf8_bytes_and_two_utf16_code_units() {
    assert_eq!("😀".encode_utf16().count(), 2);
    assert_roundtrip("😀", &[0x04, 0xf0, 0x9f, 0x98, 0x80]);
}

#[test]
fn nul_uses_standard_utf8_instead_of_modified_utf8() {
    assert_roundtrip("\0", &[0x01, 0x00]);
}

#[test]
fn maximum_utf16_length_is_accepted() {
    let value = PrefixedString("a".repeat(PrefixedString::MAX_UTF16_CODE_UNITS));
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    assert_eq!(&encoded[..3], &[0xff, 0xff, 0x01]);
    assert_eq!(encoded.len(), PrefixedString::MAX_UTF16_CODE_UNITS + 3);

    let mut input = encoded.as_slice();
    assert_eq!(PrefixedString::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn maximum_utf8_byte_length_is_accepted() {
    let value = PrefixedString("中".repeat(PrefixedString::MAX_UTF16_CODE_UNITS));
    assert_eq!(value.0.len(), PrefixedString::MAX_BYTES);

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();
    assert_eq!(PrefixedString::decode(&mut input).unwrap(), value);
}

#[test]
fn encode_rejects_too_many_utf16_code_units_before_writing() {
    let value = PrefixedString("a".repeat(PrefixedString::MAX_UTF16_CODE_UNITS + 1));
    let mut output = Vec::new();
    let error = value.encode(&mut output).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooManyUtf16CodeUnits {
            max_code_units: PrefixedString::MAX_UTF16_CODE_UNITS,
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
    assert!(output.is_empty());
}

#[test]
fn encode_rejects_too_many_utf8_bytes_before_writing() {
    let value = PrefixedString("中".repeat(PrefixedString::MAX_UTF16_CODE_UNITS + 1));
    assert!(value.0.len() > PrefixedString::MAX_BYTES);

    let mut output = Vec::new();
    let error = value.encode(&mut output).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: PrefixedString::MAX_BYTES,
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
    assert!(output.is_empty());
}

#[test]
fn decode_rejects_declared_byte_length_above_limit_before_reading_payload() {
    let mut encoded = Vec::new();
    encoded
        .write_varint((PrefixedString::MAX_BYTES + 1) as i32)
        .unwrap();

    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::StringTooLong {
            max_bytes: PrefixedString::MAX_BYTES,
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn decode_rejects_negative_length() {
    let encoded = [0xff, 0xff, 0xff, 0xff, 0x0f];
    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::NegativeLength { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn truncated_payload_is_unexpected_eof() {
    let encoded = [0x05, b'H', b'e'];
    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
    assert_eq!(
        error.io_error().unwrap().kind(),
        io::ErrorKind::UnexpectedEof
    );
}

#[test]
fn invalid_utf8_is_rejected() {
    let encoded = [0x02, 0xc3, 0x28];
    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidUtf8 {
            valid_up_to: 0,
            error_len: Some(1),
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn decode_rejects_too_many_utf16_code_units() {
    let payload = "a".repeat(PrefixedString::MAX_UTF16_CODE_UNITS + 1);
    let mut encoded = Vec::new();
    encoded.write_varint(payload.len() as i32).unwrap();
    encoded.extend_from_slice(payload.as_bytes());

    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooManyUtf16CodeUnits {
            max_code_units: PrefixedString::MAX_UTF16_CODE_UNITS,
        })
    );
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}

struct PayloadFailingWriter {
    writes: usize,
}

impl Write for PayloadFailingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.writes == 0 {
            self.writes += 1;
            Ok(buffer.len())
        } else {
            Err(io::Error::other("payload write failed"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn payload_write_error_is_attributed_to_string() {
    let mut writer = PayloadFailingWriter { writes: 0 };
    let error = PrefixedString("a".to_owned())
        .encode(&mut writer)
        .unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 1);
    assert_eq!(error.io_error().unwrap().kind(), io::ErrorKind::Other);
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
fn partial_prefix_write_preserves_varint_and_string_context() {
    let value = PrefixedString("a".repeat(128));
    let error = value
        .encode(&mut FailAfterWriter { remaining: 1 })
        .unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.context(), Some(CodecKind::String));
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 1);
    assert!(error.to_string().contains("while processing String"));
}

#[test]
fn partial_payload_write_includes_prefix_and_payload_progress() {
    let value = PrefixedString("hello".to_owned());
    let error = value
        .encode(&mut FailAfterWriter { remaining: 3 })
        .unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.context(), None);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 3);
}

#[test]
fn truncated_prefix_preserves_varint_and_string_context() {
    let encoded = [0x80];
    let error = PrefixedString::decode(&mut encoded.as_slice()).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.context(), Some(CodecKind::String));
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 1);
}

struct PartialErrorReader {
    input: &'static [u8],
}

impl io::Read for PartialErrorReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.input.is_empty() {
            return Err(io::Error::other("injected payload read failure"));
        }

        let read = buffer.len().min(self.input.len());
        buffer[..read].copy_from_slice(&self.input[..read]);
        self.input = &self.input[read..];
        Ok(read)
    }
}

#[test]
fn partial_payload_read_includes_prefix_and_payload_progress() {
    let mut reader = PartialErrorReader {
        input: &[0x05, b'H', b'e'],
    };
    let error = PrefixedString::decode(&mut reader).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::String);
    assert_eq!(error.context(), None);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 3);
}
