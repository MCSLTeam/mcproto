use std::io::{Read, Write};
use crate::CodecError;

pub trait VarIntWrite: Write {
    #[inline]
    fn write_varint(&mut self, value: i32) -> Result<(), CodecError> {
        let mut value = value as u32;
        for i in 0..5 {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            let has_next = value != 0 && i < 4;
            let byte = if has_next { byte | 0x80 } else { byte };

            self.write_all(&[byte])?;

            if !has_next {
                return Ok(());
            }
        }
        Err(CodecError::VarIntOverflow)
    }
}

pub trait VarIntRead: Read {
    #[inline]
    fn read_varint(&mut self) -> Result<i32, CodecError> {
        let mut result = 0u32;
        let mut shift = 0;

        for _ in 0..5 {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            let byte = buf[0];

            let value = (byte & 0x7F) as u32;
            result |= value << shift;

            if (byte & 0x80) == 0 {
                return Ok(result as i32);
            }

            shift += 7;
        }
        Err(CodecError::VarIntOverflow)
    }
}

impl<R: Read> VarIntRead for R {}
impl<W: Write> VarIntWrite for W {}