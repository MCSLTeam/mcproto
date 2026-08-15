//! Reading and writing [Minecraft protocol VarInt] values.
//!
//! [Minecraft protocol VarInt]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong

use std::io::{Read, Write};

use crate::error::{CodecError, CodecKind, InvalidEncodingReason};
use crate::io::{read_exact_counted, write_all_counted};

/// Extension methods for writing [Minecraft protocol VarInt] values.
///
/// This trait is implemented for every [`Write`] type.
///
/// # Example
///
/// ```
/// use mcproto_codec::varint::VarIntWrite;
///
/// let mut output = Vec::new();
/// output.write_varint(25565)?;
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// [Minecraft protocol VarInt]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
pub trait VarIntWrite: Write {
    /// Writes `value` to this writer as a VarInt.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the underlying writer fails. The error's
    /// byte count reports how much of this value was written successfully.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn write_varint(&mut self, value: i32) -> Result<(), CodecError> {
        self.write_varint_with_size(value).map(|_| ())
    }

    /// Writes `value` as a VarInt and returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the underlying writer fails. The error's
    /// byte count reports how much of this value was written successfully.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn write_varint_with_size(&mut self, value: i32) -> Result<usize, CodecError> {
        let mut value = value as u32;
        let mut bytes_processed = 0;

        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            let has_next = value != 0;
            let byte = if has_next { byte | 0x80 } else { byte };

            write_all_counted(self, &[byte], CodecKind::VarInt, bytes_processed)?;
            bytes_processed += 1;

            if !has_next {
                return Ok(bytes_processed);
            }
        }
    }
}

/// Extension methods for reading [Minecraft protocol VarInt] values.
///
/// This trait is implemented for every [`Read`] type.
///
/// # Example
///
/// ```
/// use mcproto_codec::varint::{VarIntRead, VarIntWrite};
///
/// let mut encoded = Vec::new();
/// encoded.write_varint(25565)?;
///
/// let value = encoded.as_slice().read_varint()?;
/// assert_eq!(value, 25565);
///
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// [Minecraft protocol VarInt]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
pub trait VarIntRead: Read {
    /// Reads and returns one VarInt from this reader.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the input ends early, the underlying reader
    /// fails, or the input is not a valid VarInt.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn read_varint(&mut self) -> Result<i32, CodecError> {
        self.read_varint_with_size().map(|(value, _)| value)
    }

    /// Reads one VarInt and returns its value and encoded size in bytes.
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if the input ends early, the underlying reader
    /// fails, or the input is not a valid VarInt.
    ///
    /// [`CodecError`]: crate::error::CodecError
    #[inline]
    fn read_varint_with_size(&mut self) -> Result<(i32, usize), CodecError> {
        let mut result = 0u32;
        let mut shift = 0;

        for i in 0..5 {
            let mut buf = [0u8; 1];
            read_exact_counted(self, &mut buf, CodecKind::VarInt, i)?;
            let byte = buf[0];

            if i == 4 {
                if (byte & 0x80) != 0 {
                    return Err(CodecError::invalid_encoding(
                        CodecKind::VarInt,
                        i + 1,
                        InvalidEncodingReason::TooLong { max_bytes: 5 },
                    ));
                }
            }

            let value = (byte & 0x7F) as u32;
            result |= value << shift;

            if (byte & 0x80) == 0 {
                return Ok((result as i32, i + 1));
            }

            shift += 7;
        }

        unreachable!("the fifth VarInt byte always terminates or returns an error")
    }
}

impl<R: Read> VarIntRead for R {}
impl<W: Write> VarIntWrite for W {}
