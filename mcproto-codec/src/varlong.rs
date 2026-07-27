use std::io::{Read, Write};

use crate::error::{CodecError, CodecKind, InvalidEncodingReason};
use crate::io::{read_exact_counted, write_all_counted};

pub trait VarLongWrite: Write {
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

pub trait VarLongRead: Read {
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
