use crate::TypeCodec;
use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
};
use std::io::{Read, Write};

/// True is encoded as 0x01, false as 0x00.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Boolean(pub bool);

impl TypeCodec for Boolean {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let byte = if self.0 { 1u8 } else { 0u8 };
        write_all_counted(writer, &[byte], CodecKind::Boolean, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 1];
        read_exact_counted(reader, &mut buf, CodecKind::Boolean, 0)?;
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
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::Byte, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 1];
        read_exact_counted(reader, &mut bytes, CodecKind::Byte, 0)?;
        Ok(Self(i8::from_be_bytes(bytes)))
    }
}

/// Unsigned 8-bit integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedByte(pub u8);

impl TypeCodec for UnsignedByte {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::UnsignedByte, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 1];
        read_exact_counted(reader, &mut bytes, CodecKind::UnsignedByte, 0)?;
        Ok(Self(u8::from_be_bytes(bytes)))
    }
}

/// Signed 16-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Short(pub i16);

impl TypeCodec for Short {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::Short, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 2];
        read_exact_counted(reader, &mut bytes, CodecKind::Short, 0)?;
        Ok(Self(i16::from_be_bytes(bytes)))
    }
}

/// Unsigned 16-bit integer, big-endian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedShort(pub u16);

impl TypeCodec for UnsignedShort {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::UnsignedShort, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 2];
        read_exact_counted(reader, &mut bytes, CodecKind::UnsignedShort, 0)?;
        Ok(Self(u16::from_be_bytes(bytes)))
    }
}

/// Signed 32-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Int(pub i32);

impl TypeCodec for Int {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::Int, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 4];
        read_exact_counted(reader, &mut bytes, CodecKind::Int, 0)?;
        Ok(Self(i32::from_be_bytes(bytes)))
    }
}

/// Signed 64-bit integer, big-endian two's complement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Long(pub i64);

impl TypeCodec for Long {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.to_be_bytes(), CodecKind::Long, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 8];
        read_exact_counted(reader, &mut bytes, CodecKind::Long, 0)?;
        Ok(Self(i64::from_be_bytes(bytes)))
    }
}

/// A standard UTF-8 string prefixed by its byte length as a VarInt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrefixedString(pub String);

impl PrefixedString {
    pub const MAX_UTF16_CODE_UNITS: usize = 0x7fff;
    pub const MAX_BYTES: usize = Self::MAX_UTF16_CODE_UNITS * 3;

    fn validate_value(
        value: &str,
        operation: CodecOperation,
        bytes_processed: usize,
    ) -> Result<(), CodecError> {
        let bytes = value.as_bytes();
        if bytes.len() > Self::MAX_BYTES {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::String,
                operation,
                bytes_processed,
                InvalidEncodingReason::StringTooLong {
                    max_bytes: Self::MAX_BYTES,
                },
            ));
        }

        if value.encode_utf16().count() > Self::MAX_UTF16_CODE_UNITS {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::String,
                operation,
                bytes_processed,
                InvalidEncodingReason::TooManyUtf16CodeUnits {
                    max_code_units: Self::MAX_UTF16_CODE_UNITS,
                },
            ));
        }

        Ok(())
    }

    fn encode_value(value: &str, writer: &mut impl Write) -> Result<(), CodecError> {
        Self::validate_value(value, CodecOperation::Write, 0)?;

        let bytes = value.as_bytes();
        let prefix_size = writer
            .write_varint_with_size(bytes.len() as i32)
            .map_err(|error| error.with_context(CodecKind::String))?;
        write_all_counted(writer, bytes, CodecKind::String, prefix_size)
    }
}

impl TypeCodec for PrefixedString {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        Self::encode_value(&self.0, writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (byte_length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::String))?;
        let byte_length = usize::try_from(byte_length).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::String,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: byte_length },
            )
        })?;

        if byte_length > Self::MAX_BYTES {
            return Err(CodecError::invalid_encoding(
                CodecKind::String,
                prefix_size,
                InvalidEncodingReason::StringTooLong {
                    max_bytes: Self::MAX_BYTES,
                },
            ));
        }

        let mut bytes = vec![0; byte_length];
        read_exact_counted(reader, &mut bytes, CodecKind::String, prefix_size)?;

        let value = String::from_utf8(bytes).map_err(|error| {
            let utf8_error = error.utf8_error();
            CodecError::invalid_encoding(
                CodecKind::String,
                prefix_size + byte_length,
                InvalidEncodingReason::InvalidUtf8 {
                    valid_up_to: utf8_error.valid_up_to(),
                    error_len: utf8_error.error_len(),
                },
            )
        })?;
        Self::validate_value(&value, CodecOperation::Read, prefix_size + byte_length)?;
        Ok(Self(value))
    }
}

/// Encoded as a String with max length of 32 767.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Identifier(pub String);

impl Identifier {
    pub const MAX_UTF16_CODE_UNITS: usize = PrefixedString::MAX_UTF16_CODE_UNITS;
    pub const MAX_BYTES: usize = PrefixedString::MAX_BYTES;
    pub const MAX_ENCODED_BYTES: usize = Self::MAX_BYTES + 3;
}

impl TypeCodec for Identifier {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        PrefixedString::encode_value(&self.0, writer)
            .map_err(|error| error.with_context(CodecKind::Identifier))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        PrefixedString::decode(reader)
            .map(|value| Self(value.0))
            .map_err(|error| error.with_context(CodecKind::Identifier))
    }
}
