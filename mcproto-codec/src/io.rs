//! Counted I/O helpers for protocol codecs.
//!
//! These functions provide the completion guarantees of [`Read::read_exact`]
//! and [`Write::write_all`] while attaching codec metadata and byte progress to
//! any [`CodecError`] they return.

use std::io::{self, Read, Write};

use crate::error::{CodecError, CodecKind};

/// Fills `buffer` from `reader` and tracks progress for codec errors.
///
/// `codec` identifies the codec performing the read. `bytes_processed` is the
/// number of bytes that codec processed before this buffer; bytes read into
/// `buffer` are added to that base when an error is reported.
///
/// Operations interrupted with [`io::ErrorKind::Interrupted`] are retried. If
/// the reader reaches the end of its input before filling `buffer`, the result
/// is a [`CodecErrorKind::UnexpectedEof`] error.
///
/// # Example
///
/// ```
/// use mcproto_codec::{
///     error::CodecKind,
///     io::read_exact_counted,
/// };
///
/// let mut input = [0x12, 0x34].as_slice();
/// let mut bytes = [0; 2];
/// read_exact_counted(&mut input, &mut bytes, CodecKind::Short, 0)?;
/// assert_eq!(bytes, [0x12, 0x34]);
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns a [`CodecError`] if the reader reaches an unexpected end of input
/// or reports another I/O error.
///
/// [`CodecErrorKind::UnexpectedEof`]: crate::error::CodecErrorKind::UnexpectedEof
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

/// Writes all of `buffer` to `writer` and tracks progress for codec errors.
///
/// `codec` identifies the codec performing the write. `bytes_processed` is the
/// number of bytes that codec processed before this buffer; bytes written from
/// `buffer` are added to that base when an error is reported.
///
/// Operations interrupted with [`io::ErrorKind::Interrupted`] are retried. A
/// successful write of zero bytes while data remains is reported as an
/// [`io::ErrorKind::WriteZero`] source error.
///
/// # Example
///
/// ```
/// use mcproto_codec::{
///     error::CodecKind,
///     io::write_all_counted,
/// };
///
/// let mut output = Vec::new();
/// write_all_counted(&mut output, &[0x12, 0x34], CodecKind::Short, 0)?;
/// assert_eq!(output, [0x12, 0x34]);
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// # Errors
///
/// Returns a [`CodecError`] if the writer cannot accept the complete buffer.
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
