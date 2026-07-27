use std::io::{self, Read, Write};

use crate::error::{CodecError, CodecKind};

#[inline]
pub fn read_exact_counted<R: Read + ?Sized>(
    reader: &mut R,
    buffer: &mut [u8],
    codec: CodecKind,
    bytes_processed: usize,
) -> Result<(), CodecError> {
    let mut current = 0;

    while current < buffer.len() {
        match reader.read(&mut buffer[current..]) {
            Ok(0) => {
                let error = io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to fill the whole buffer",
                );
                return Err(CodecError::from_read_error(
                    codec,
                    bytes_processed + current,
                    error,
                ));
            }
            Ok(read) => current += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                return Err(CodecError::from_read_error(
                    codec,
                    bytes_processed + current,
                    error,
                ));
            }
        }
    }

    Ok(())
}

#[inline]
pub fn write_all_counted<W: Write + ?Sized>(
    writer: &mut W,
    buffer: &[u8],
    codec: CodecKind,
    bytes_processed: usize,
) -> Result<(), CodecError> {
    let mut current = 0;

    while current < buffer.len() {
        match writer.write(&buffer[current..]) {
            Ok(0) => {
                let error =
                    io::Error::new(io::ErrorKind::WriteZero, "failed to write the whole buffer");
                return Err(CodecError::from_write_error(
                    codec,
                    bytes_processed + current,
                    error,
                ));
            }
            Ok(written) => current += written,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                return Err(CodecError::from_write_error(
                    codec,
                    bytes_processed + current,
                    error,
                ));
            }
        }
    }

    Ok(())
}
