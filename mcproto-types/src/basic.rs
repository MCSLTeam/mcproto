use derive_more::with_trait::{Into, Deref, DerefMut, From};
use crate::{Codec, TypeCodecError};
use mcproto_codec::varint::*;
use mcproto_codec::varlong::*;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, From, Into, Deref)]
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
#[derive(Debug, Clone, PartialEq, From, Into, Deref)]
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

// String
impl Codec for String {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        let bytes = self.as_bytes();
        let len = i32::try_from(bytes.len())
            .map_err(|_| TypeCodecError::InvalidStringLength(bytes.len()))?;
        buf.write_varint(len)?;
        buf.extend_from_slice(bytes);
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let len = buf.read_varint()?;
        let len = usize::try_from(len)
            .map_err(|_| TypeCodecError::InvalidStringLength(len as usize))?;
        let bytes = buf
            .get(..len)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), len))?;
        *buf = &buf[len..];
        String::from_utf8(bytes.to_vec())
            .map_err(|_| TypeCodecError::InvalidUtf8)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref)]
pub struct Identifier(pub String);
impl Codec for Identifier {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.0.encode(buf)
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        String::decode(buf).map(Identifier)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct VarInt(pub i32);

impl Codec for VarInt {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        Ok(buf.write_varint(self.0)?)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(VarInt(buf.read_varint()?))
    }
}

// VarLong
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct VarLong(pub i64);

impl Codec for VarLong {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        Ok(buf.write_varlong(self.0)?)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(VarLong(buf.read_varlong()?))
    }
}