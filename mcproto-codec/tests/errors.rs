use std::io::{self, Read, Write};

use mcproto_codec::{
    error::{CodecErrorKind, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
    varlong::{VarLongRead, VarLongWrite},
};

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

#[test]
fn varint_size_methods_report_encoded_size() {
    let mut encoded = Vec::new();
    assert_eq!(encoded.write_varint_with_size(25565).unwrap(), 3);

    let mut input = encoded.as_slice();
    assert_eq!(input.read_varint_with_size().unwrap(), (25565, 3));
}

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

#[test]
fn counted_write_retries_interrupted_operations() {
    let mut writer = InterruptedWriter {
        output: Vec::new(),
        interrupted: false,
    };

    write_all_counted(&mut writer, &[0x12, 0x34], CodecKind::Short, 0).unwrap();
    assert_eq!(writer.output, [0x12, 0x34]);
}

struct ZeroWriter;

impl Write for ZeroWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn counted_write_maps_zero_to_write_zero_at_the_base_offset() {
    let error = write_all_counted(&mut ZeroWriter, &[1], CodecKind::Byte, 7).unwrap_err();

    assert_eq!(error.kind(), CodecErrorKind::Io);
    assert_eq!(error.bytes_processed(), 7);
    assert_eq!(error.io_error().unwrap().kind(), io::ErrorKind::WriteZero);
}
