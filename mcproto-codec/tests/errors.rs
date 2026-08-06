//! Integration tests for codec error classification and byte-progress tracking.
//!
//! The fixtures inject short writes, interrupted operations, truncated input,
//! and malformed encodings so the public error metadata can be checked at the
//! point where each failure occurs.

use std::io::{self, Read, Write};

use mcproto_codec::{
    error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
    varlong::{VarLongRead, VarLongWrite},
};

// Accepts exactly `remaining` bytes, then injects a persistent I/O failure.
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

// Write failures must retain the codec, operation, and exact progress.
#[test]
fn varint_immediate_write_error_reports_zero() {
    let error = FailAfterWriter { remaining: 0 }
        .write_varint(1)
        .unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.operation(), CodecOperation::Write);
    assert_eq!(error.bytes_processed(), 0);
}

#[test]
fn varint_partial_write_error_reports_exact_progress() {
    let error = FailAfterWriter { remaining: 2 }
        .write_varint(25565)
        .unwrap_err();

    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.bytes_processed(), 2);
}

#[test]
fn varlong_partial_write_error_reports_exact_progress() {
    let error = FailAfterWriter { remaining: 7 }
        .write_varlong(-1)
        .unwrap_err();

    assert_eq!(error.codec(), CodecKind::VarLong);
    assert_eq!(error.bytes_processed(), 7);
}

// Truncated reads are classified separately from other I/O errors.
#[test]
fn varint_truncated_read_reports_exact_progress() {
    let mut input = [0x80, 0x80].as_slice();
    let error = input.read_varint().unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::VarInt);
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), 2);
}

#[test]
fn varlong_truncated_read_reports_exact_progress() {
    let mut input = [0x80, 0x80, 0x80].as_slice();
    let error = input.read_varlong().unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::UnexpectedEof);
    assert_eq!(error.codec(), CodecKind::VarLong);
    assert_eq!(error.bytes_processed(), 3);
}

// The size-returning VarInt APIs agree on the encoded length.
#[test]
fn varint_size_methods_report_encoded_size() {
    let mut encoded = Vec::new();
    assert_eq!(encoded.write_varint_with_size(25565).unwrap(), 3);

    let mut input = encoded.as_slice();
    assert_eq!(input.read_varint_with_size().unwrap(), (25565, 3));
}

// The size-returning VarLong APIs agree on the encoded length.
#[test]
fn varlong_size_methods_report_encoded_size() {
    let mut encoded = Vec::new();
    assert_eq!(encoded.write_varlong_with_size(-1).unwrap(), 10);

    let mut input = encoded.as_slice();
    assert_eq!(input.read_varlong_with_size().unwrap(), (-1, 10));
}

// Malformed terminal bytes report both the reason and consumed byte count.
#[test]
fn varint_rejects_continuation_in_fifth_byte() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x80].as_slice();
    let error = input.read_varint().unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooLong { max_bytes: 5 })
    );
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn varint_rejects_out_of_range_terminal_bits() {
    let mut input = [0xff, 0xff, 0xff, 0xff, 0x10].as_slice();
    let error = input.read_varint().unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ValueOutOfRange {
            terminal_byte: 0x10,
            allowed_mask: 0x0f,
        })
    );
    assert_eq!(error.bytes_processed(), 5);
}

#[test]
fn varlong_rejects_continuation_in_tenth_byte() {
    let mut encoded = [0xff; 10];
    encoded[9] = 0x80;
    let error = encoded.as_slice().read_varlong().unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::TooLong { max_bytes: 10 })
    );
    assert_eq!(error.bytes_processed(), 10);
}

#[test]
fn varlong_rejects_out_of_range_terminal_bits() {
    let mut encoded = [0xff; 10];
    encoded[9] = 0x02;
    let error = encoded.as_slice().read_varlong().unwrap_err();

    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ValueOutOfRange {
            terminal_byte: 0x02,
            allowed_mask: 0x01,
        })
    );
    assert_eq!(error.bytes_processed(), 10);
}

// Produces one Interrupted error before delegating all subsequent reads.
struct InterruptedReader<'a> {
    input: &'a [u8],
    interrupted: bool,
}

impl Read for InterruptedReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }

        self.input.read(buffer)
    }
}

// Counted reads must retry transient interruptions without changing progress.
#[test]
fn counted_read_retries_interrupted_operations() {
    let mut reader = InterruptedReader {
        input: &[0x12, 0x34],
        interrupted: false,
    };
    let mut output = [0; 2];

    read_exact_counted(&mut reader, &mut output, CodecKind::Short, 0).unwrap();
    assert_eq!(output, [0x12, 0x34]);
}

// Produces one Interrupted error before accepting all subsequent writes.
struct InterruptedWriter {
    output: Vec<u8>,
    interrupted: bool,
}

impl Write for InterruptedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }

        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Counted writes must retry transient interruptions without duplicating data.
#[test]
fn counted_write_retries_interrupted_operations() {
    let mut writer = InterruptedWriter {
        output: Vec::new(),
        interrupted: false,
    };

    write_all_counted(&mut writer, &[0x12, 0x34], CodecKind::Short, 0).unwrap();
    assert_eq!(writer.output, [0x12, 0x34]);
}

// Simulates a writer that cannot make progress without returning an error.
struct ZeroWriter;

impl Write for ZeroWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// A zero-length write becomes WriteZero at the caller-provided base offset.
#[test]
fn counted_write_maps_zero_to_write_zero_at_the_base_offset() {
    let error = write_all_counted(&mut ZeroWriter, &[1], CodecKind::Byte, 7).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.bytes_processed(), 7);
    assert_eq!(error.io_error().unwrap().kind(), io::ErrorKind::WriteZero);
}
