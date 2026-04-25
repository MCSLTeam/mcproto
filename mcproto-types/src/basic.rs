use derive_more::with_trait::{Into, Deref, DerefMut, From};
use crate::{Codec, TypeCodecError};

// 基础类型

// Boolean:

impl Codec for bool {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.push(*self as u8);
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        if buf.is_empty() {
            return Err(TypeCodecError::EmptyBuffer);
        }
        let byte = buf[0];
        *buf = &buf[1..];
        match byte {
            0x00 => Ok(false),
            0x01 => Ok(true),
            other => Err(TypeCodecError::InvalidBoolean(other)),
        }
    }
}


// Byte
#[derive(Into, From, Deref, DerefMut)]
pub struct Byte(pub i8);
impl Codec for Byte {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.push(self.0 as u8);  // i8 → u8，二进制补码不变
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        if buf.is_empty() {
            return Err(TypeCodecError::EmptyBuffer);
        }
        let byte = buf[0];
        *buf = &buf[1..];
        Ok(Byte(byte as i8))  // u8 → i8，二进制补码不变
    }
}

// Unsigned Byte
#[derive(Into, From, Deref, DerefMut)]
pub struct UnsignedByte(pub u8);
impl Codec for UnsignedByte {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.push(self.0);
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        if buf.is_empty() {
            return Err(TypeCodecError::EmptyBuffer);
        }
        let byte = buf[0];
        *buf = &buf[1..];
        Ok(UnsignedByte(byte))
    }
}

// Short
#[derive(Into, From, Deref, DerefMut)]
pub struct Short(pub i16);
impl Codec for Short {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 2] = buf
            .get(..2)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 2))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 2))?;
        *buf = &buf[2..];
        Ok(Short(i16::from_be_bytes(bytes)))
    }
}
// Unsigned Short
#[derive(Into, From, Deref, DerefMut)]
pub struct UnsignedShort(pub u16);
impl Codec for UnsignedShort {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 2] = buf
            .get(..2)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 2))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 2))?;
        *buf = &buf[2..];
        Ok(UnsignedShort(u16::from_be_bytes(bytes)))
    }
}
// Int
#[derive(Into, From, Deref, DerefMut)]
pub struct Int(pub i32);

impl Codec for Int {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 4] = buf
            .get(..4)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 4))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 4))?;
        *buf = &buf[4..];
        Ok(Int(i32::from_be_bytes(bytes)))
    }
}
// Long
#[derive(Into, From, Deref, DerefMut)]
pub struct Long(pub i64);

impl Codec for Long {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 8] = buf
            .get(..8)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 8))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 8))?;
        *buf = &buf[8..];
        Ok(Long(i64::from_be_bytes(bytes)))
    }
}
// Float
#[derive(Into, From, Deref, DerefMut)]
pub struct Float(pub f32);

impl Codec for Float {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_bits().to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 4] = buf
            .get(..4)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 4))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 4))?;
        *buf = &buf[4..];
        Ok(Float(f32::from_be_bytes(bytes)))
    }
}
// Double
#[derive(Into, From, Deref, DerefMut)]
pub struct Double(pub f64);

impl Codec for Double {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0.to_bits().to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 8] = buf
            .get(..8)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 8))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 8))?;
        *buf = &buf[8..];
        Ok(Double(f64::from_be_bytes(bytes)))
    }
}

