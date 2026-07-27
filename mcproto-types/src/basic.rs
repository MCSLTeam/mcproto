use crate::TypeCodec;
use mcproto_codec::error::{CodecError, CodecKind, InvalidEncodingReason};
use std::io::{Read, Write};

/// True is encoded as 0x01, false as 0x00.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Boolean(pub bool);

impl TypeCodec for Boolean {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let byte = if self.0 { 1u8 } else { 0u8 };
        writer
            .write_all(&[byte])
            .map_err(|error| CodecError::from_write_error(CodecKind::Boolean, 1, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 1];
        reader
            .read_exact(&mut buf)
            .map_err(|error| CodecError::from_read_error(CodecKind::Boolean, 0, error))?;
        match buf[0] {
            0 => Ok(Boolean(false)),
            1 => Ok(Boolean(true)),
            _ => Err(CodecError::invalid_encoding(
                CodecKind::Boolean,
                1,
                InvalidEncodingReason::InvalidBooleanValue { value: buf[0] },
            )),
        }
    }
}

/// Signed 8-bit integer, two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Byte(pub i8);

impl TypeCodec for Byte {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::Byte, 1, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 1];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::Byte, 0, error))?;
        Ok(Self(i8::from_be_bytes(bytes)))
    }
}

/// Unsigned 8-bit integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedByte(pub u8);

impl TypeCodec for UnsignedByte {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::UnsignedByte, 1, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 1];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::UnsignedByte, 0, error))?;
        Ok(Self(u8::from_be_bytes(bytes)))
    }
}

/// Signed 16-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Short(pub i16);

impl TypeCodec for Short {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::Short, 2, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 2];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::Short, 0, error))?;
        Ok(Self(i16::from_be_bytes(bytes)))
    }
}

/// Unsigned 16-bit integer, big-endian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedShort(pub u16);

impl TypeCodec for UnsignedShort {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::UnsignedShort, 2, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 2];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::UnsignedShort, 0, error))?;
        Ok(Self(u16::from_be_bytes(bytes)))
    }
}

/// Signed 32-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Int(pub i32);

impl TypeCodec for Int {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::Int, 4, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 4];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::Int, 0, error))?;
        Ok(Self(i32::from_be_bytes(bytes)))
    }
}

/// Signed 64-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Long(pub i64);

impl TypeCodec for Long {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer
            .write_all(&self.0.to_be_bytes())
            .map_err(|error| CodecError::from_write_error(CodecKind::Long, 8, error))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 8];
        reader
            .read_exact(&mut bytes)
            .map_err(|error| CodecError::from_read_error(CodecKind::Long, 0, error))?;
        Ok(Self(i64::from_be_bytes(bytes)))
    }
}
