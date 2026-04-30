use std::str;

use crate::{NbtError, TagKind};

#[derive(Debug, Clone, Copy)]
pub struct NbtReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> NbtReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buf
    }

    pub fn advance(&mut self, len: usize) -> Result<(), NbtError> {
        self.ensure_available(len)?;
        self.pos += len;
        Ok(())
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], NbtError> {
        self.ensure_available(len)?;
        let start = self.pos;
        self.pos += len;
        Ok(&self.buf[start..self.pos])
    }

    pub fn peek_u8(&self) -> Result<u8, NbtError> {
        self.ensure_available(1)?;
        Ok(self.buf[self.pos])
    }

    pub fn read_u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, NbtError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_i16(&mut self) -> Result<i16, NbtError> {
        let bytes: [u8; 2] = self
            .read_bytes(2)?
            .try_into()
            .expect("read_bytes returned incorrect length");
        Ok(i16::from_be_bytes(bytes))
    }

    pub fn read_u16(&mut self) -> Result<u16, NbtError> {
        let bytes: [u8; 2] = self
            .read_bytes(2)?
            .try_into()
            .expect("read_bytes returned incorrect length");
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_i32(&mut self) -> Result<i32, NbtError> {
        let bytes: [u8; 4] = self
            .read_bytes(4)?
            .try_into()
            .expect("read_bytes returned incorrect length");
        Ok(i32::from_be_bytes(bytes))
    }

    pub fn read_i64(&mut self) -> Result<i64, NbtError> {
        let bytes: [u8; 8] = self
            .read_bytes(8)?
            .try_into()
            .expect("read_bytes returned incorrect length");
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn read_f32(&mut self) -> Result<f32, NbtError> {
        Ok(f32::from_bits(self.read_i32()? as u32))
    }

    pub fn read_f64(&mut self) -> Result<f64, NbtError> {
        Ok(f64::from_bits(self.read_i64()? as u64))
    }

    pub fn read_tag_kind(&mut self) -> Result<TagKind, NbtError> {
        let offset = self.pos;
        let value = self.read_u8()?;
        TagKind::try_from(value).map_err(|value| NbtError::InvalidTagKind { offset, value })
    }

    pub fn peek_tag_kind(&self) -> Result<TagKind, NbtError> {
        let offset = self.pos;
        let value = self.peek_u8()?;
        TagKind::try_from(value).map_err(|value| NbtError::InvalidTagKind { offset, value })
    }

    pub fn read_string(&mut self) -> Result<&'a str, NbtError> {
        let len = usize::from(self.read_u16()?);
        let offset = self.pos;
        let bytes = self.read_bytes(len)?;
        str::from_utf8(bytes).map_err(|_| NbtError::InvalidUtf8 { offset })
    }

    pub fn peek_string(&self) -> Result<&'a str, NbtError> {
        let mut reader = *self;
        reader.read_string()
    }

    fn ensure_available(&self, needed: usize) -> Result<(), NbtError> {
        let remaining = self.remaining();
        if remaining < needed {
            return Err(NbtError::UnexpectedEof {
                offset: self.pos,
                needed,
                remaining,
            });
        }
        Ok(())
    }
}

