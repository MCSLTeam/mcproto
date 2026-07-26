use std::io::{Read, Write};

use crate::error::{CodecError, CodecKind, InvalidEncodingReason};

pub trait VarLongWrite: Write {
    #[inline]
    fn write_varlong(&mut self, value: i64) -> Result<(), CodecError> {
        let mut value = value as u64;
        for i in 0..10 {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            let has_next = value != 0;
            let byte = if has_next { byte | 0x80 } else { byte };

            self.write_all(&[byte])
                .map_err(|e| CodecError::from_write_error(CodecKind::VarLong, i + 1, e))?;

            if !has_next {
                return Ok(());
            }
        }
        Err(CodecError::invalid_encoding(
            CodecKind::VarLong,
            10,
            InvalidEncodingReason::TooLong { max_bytes: 10 },
        ))
    }
}

pub trait VarLongRead: Read {
    #[inline]
    fn read_varlong(&mut self) -> Result<i64, CodecError> {
        let mut result = 0u64;
        let mut shift = 0;

        for i in 0..10 {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)
                .map_err(|e| CodecError::from_read_error(CodecKind::VarLong, i, e))?;
            let byte = buf[0];

            let value = (byte & 0x7F) as u64;
            result |= value << shift;

            if (byte & 0x80) == 0 {
                return Ok(result as i64);
            }

            shift += 7;
        }
        Err(CodecError::invalid_encoding(
            CodecKind::VarLong,
            10,
            InvalidEncodingReason::TooLong { max_bytes: 10 },
        ))
    }
}

impl<R: Read> VarLongRead for R {}
impl<W: Write> VarLongWrite for W {}
