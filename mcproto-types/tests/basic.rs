use std::io::{self, Read, Write};

use mcproto_codec::error::{
    CodecError, CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason,
};
use mcproto_types::{
    TypeCodec,
    basic::{Boolean, Byte, Int, Long, Short, UnsignedByte, UnsignedShort},
};

macro_rules! codec_case {
    ($name:ident, $type:ident, $value:expr, [$($byte:expr),+ $(,)?]) => {
        #[test]
        fn $name() {
            let expected = [$($byte),+];

            let mut encoded = Vec::new();
            $type($value).encode(&mut encoded).unwrap();
            assert_eq!(encoded, expected);

            let mut input = expected.as_slice();
            assert_eq!($type::decode(&mut input).unwrap(), $type($value));
            assert!(input.is_empty());
        }
    };
}

codec_case!(boolean_false, Boolean, false, [0x00]);
codec_case!(boolean_true, Boolean, true, [0x01]);

codec_case!(byte_min, Byte, i8::MIN, [0x80]);
codec_case!(byte_negative_one, Byte, -1, [0xff]);
codec_case!(byte_zero, Byte, 0, [0x00]);
codec_case!(byte_max, Byte, i8::MAX, [0x7f]);

codec_case!(unsigned_byte_zero, UnsignedByte, 0, [0x00]);
codec_case!(unsigned_byte_example, UnsignedByte, 0xab, [0xab]);
codec_case!(unsigned_byte_max, UnsignedByte, u8::MAX, [0xff]);

codec_case!(short_min, Short, i16::MIN, [0x80, 0x00]);
codec_case!(short_negative_one, Short, -1, [0xff, 0xff]);
codec_case!(short_zero, Short, 0, [0x00, 0x00]);
codec_case!(short_example, Short, 0x1234, [0x12, 0x34]);
codec_case!(short_max, Short, i16::MAX, [0x7f, 0xff]);

codec_case!(unsigned_short_zero, UnsignedShort, 0, [0x00, 0x00]);
codec_case!(unsigned_short_example, UnsignedShort, 0xabcd, [0xab, 0xcd]);
codec_case!(unsigned_short_max, UnsignedShort, u16::MAX, [0xff, 0xff]);

codec_case!(int_min, Int, i32::MIN, [0x80, 0x00, 0x00, 0x00]);
codec_case!(int_negative_one, Int, -1, [0xff, 0xff, 0xff, 0xff]);
codec_case!(int_zero, Int, 0, [0x00, 0x00, 0x00, 0x00]);
codec_case!(int_example, Int, 0x1234_5678, [0x12, 0x34, 0x56, 0x78]);
codec_case!(int_max, Int, i32::MAX, [0x7f, 0xff, 0xff, 0xff]);

codec_case!(
    long_min,
    Long,
    i64::MIN,
    [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
);
codec_case!(
    long_negative_one,
    Long,
    -1,
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
);
codec_case!(
    long_zero,
    Long,
    0,
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
);
codec_case!(
    long_example,
    Long,
    0x0123_4567_89ab_cdef,
    [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
);
codec_case!(
    long_max,
    Long,
    i64::MAX,
    [0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
);

#[test]
fn boolean_rejects_non_boolean_values() {
    for value in [0x02, 0x7f, 0xff] {
        let encoded = [value];
        let mut input = encoded.as_slice();
        let error = Boolean::decode(&mut input).unwrap_err();

        assert_eq!(
            error.kind(),
            CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidBooleanValue { value })
        );
        assert_eq!(error.codec(), CodecKind::Boolean);
        assert_eq!(error.operation(), CodecOperation::Read);
        assert_eq!(error.bytes_processed(), 1);
        assert!(error.io_error().is_none());
    }
}

#[test]
fn decoding_consumes_only_the_value_bytes() {
    let mut input = [0x12, 0x34, 0x56, 0x78, 0xaa].as_slice();

    assert_eq!(Int::decode(&mut input).unwrap(), Int(0x1234_5678));
    assert_eq!(input, [0xaa]);
}

fn assert_error(
    error: &CodecError,
    error_kind: CodecErrorKind,
    codec_kind: CodecKind,
    operation: CodecOperation,
) {
    assert_eq!(error.kind(), error_kind);
    assert_eq!(error.codec(), codec_kind);
    assert_eq!(error.operation(), operation);
    assert!(error.io_error().is_some());
}

macro_rules! eof_case {
    ($name:ident, $type:ident, $kind:ident, [$($byte:expr),* $(,)?]) => {
        #[test]
        fn $name() {
            let mut input = [$($byte),*].as_slice();
            let error = $type::decode(&mut input).unwrap_err();

            assert_error(
                &error,
                CodecErrorKind::UnexpectedEof,
                CodecKind::$kind,
                CodecOperation::Read,
            );
            assert_eq!(
                error.io_error().unwrap().kind(),
                io::ErrorKind::UnexpectedEof
            );
        }
    };
}

eof_case!(boolean_eof, Boolean, Boolean, []);
eof_case!(byte_eof, Byte, Byte, []);
eof_case!(unsigned_byte_eof, UnsignedByte, UnsignedByte, []);
eof_case!(short_eof, Short, Short, [0x00]);
eof_case!(unsigned_short_eof, UnsignedShort, UnsignedShort, [0x00]);
eof_case!(int_eof, Int, Int, [0x00, 0x00, 0x00]);
eof_case!(
    long_eof,
    Long,
    Long,
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
);

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("read failed"))
    }
}

macro_rules! read_error_case {
    ($name:ident, $type:ident, $kind:ident) => {
        #[test]
        fn $name() {
            let error = $type::decode(&mut FailingReader).unwrap_err();

            assert_error(
                &error,
                CodecErrorKind::Io,
                CodecKind::$kind,
                CodecOperation::Read,
            );
            assert_eq!(error.io_error().unwrap().kind(), io::ErrorKind::Other);
        }
    };
}

read_error_case!(boolean_read_error, Boolean, Boolean);
read_error_case!(byte_read_error, Byte, Byte);
read_error_case!(unsigned_byte_read_error, UnsignedByte, UnsignedByte);
read_error_case!(short_read_error, Short, Short);
read_error_case!(unsigned_short_read_error, UnsignedShort, UnsignedShort);
read_error_case!(int_read_error, Int, Int);
read_error_case!(long_read_error, Long, Long);

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("write failed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

macro_rules! write_error_case {
    ($name:ident, $value:expr, $kind:ident, $size:expr) => {
        #[test]
        fn $name() {
            let error = $value.encode(&mut FailingWriter).unwrap_err();

            assert_error(
                &error,
                CodecErrorKind::Io,
                CodecKind::$kind,
                CodecOperation::Write,
            );
            assert_eq!(error.bytes_processed(), $size);
            assert_eq!(error.io_error().unwrap().kind(), io::ErrorKind::Other);
        }
    };
}

write_error_case!(boolean_write_error, Boolean(false), Boolean, 1);
write_error_case!(byte_write_error, Byte(0), Byte, 1);
write_error_case!(unsigned_byte_write_error, UnsignedByte(0), UnsignedByte, 1);
write_error_case!(short_write_error, Short(0), Short, 2);
write_error_case!(
    unsigned_short_write_error,
    UnsignedShort(0),
    UnsignedShort,
    2
);
write_error_case!(int_write_error, Int(0), Int, 4);
write_error_case!(long_write_error, Long(0), Long, 8);
