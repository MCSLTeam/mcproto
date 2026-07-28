//! Reading and writing [Minecraft protocol VarLong] values.
//!
//! [Minecraft protocol VarLong]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong

use std::io::{Read, Write};

use crate::error::{CodecError, CodecKind, InvalidEncodingReason};
use crate::io::{read_exact_counted, write_all_counted};

/// Extension methods for writing [Minecraft protocol VarLong] values.
///
/// This trait is implemented for every [`Write`] type.
///
/// # Example
///
/// ```
/// use mcproto_codec::varlong::VarLongWrite;
///
/// let mut output = Vec::new();
/// output.write_varlong(9_223_372_036_854_775_000)?;
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// [Minecraft protocol VarLong]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
pub trait VarLongWrite: Write {
    /// Writes `value` to this writer as a VarLong.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the underlying writer fails. The error's
    /// byte count reports how much of this value was written successfully.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn write_varlong(&mut self, value: i64) -> Result<(), CodecError> {
        let mut value = value as u64;
        let mut bytes_processed = 0;

        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            let has_next = value != 0;
            let byte = if has_next { byte | 0x80 } else { byte };

            write_all_counted(self, &[byte], CodecKind::VarLong, bytes_processed)?;
            bytes_processed += 1;

            if !has_next {
                return Ok(());
            }
        }
    }
}

/// Extension methods for reading [Minecraft protocol VarLong] values.
///
/// This trait is implemented for every [`Read`] type.
///
/// # Example
///
/// ```
/// use mcproto_codec::varlong::{VarLongRead, VarLongWrite};
///
/// let mut encoded = Vec::new();
/// encoded.write_varlong(9_223_372_036_854_775_000)?;
///
/// let value = encoded.as_slice().read_varlong()?;
/// assert_eq!(value, 9_223_372_036_854_775_000);
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// [Minecraft protocol VarLong]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
pub trait VarLongRead: Read {
    /// Reads and returns one VarLong from this reader.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the input ends early, the underlying reader
    /// fails, or the input is not a valid VarLong.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn read_varlong(&mut self) -> Result<i64, CodecError> {
        let mut result = 0u64;
        let mut shift = 0;

        for i in 0..10 {
            let mut buf = [0u8; 1];
            read_exact_counted(self, &mut buf, CodecKind::VarLong, i)?;
            let byte = buf[0];

            if i == 9 {
                if (byte & 0x80) != 0 {
                    return Err(CodecError::invalid_encoding(
                        CodecKind::VarLong,
                        i + 1,
                        InvalidEncodingReason::TooLong { max_bytes: 10 },
                    ));
                }
                if (byte & !0x01) != 0 {
                    return Err(CodecError::invalid_encoding(
                        CodecKind::VarLong,
                        i + 1,
                        InvalidEncodingReason::ValueOutOfRange {
                            terminal_byte: byte,
                            allowed_mask: 0x01,
                        },
                    ));
                }
            }

            let value = (byte & 0x7F) as u64;
            result |= value << shift;

            if (byte & 0x80) == 0 {
                return Ok(result as i64);
            }

            shift += 7;
        }

        unreachable!("the tenth VarLong byte always terminates or returns an error")
    }
}

impl<R: Read> VarLongRead for R {}
impl<W: Write> VarLongWrite for W {}
