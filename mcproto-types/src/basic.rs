use crate::TypeCodec;
use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
};
use std::{
    fmt,
    io::{Read, Write},
};

/// A boolean encoded as `0x00` for false or `0x01` for true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Boolean(
    /// The boolean value.
    pub bool,
);

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

/// A two's-complement signed 8-bit integer from -128 through 127.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Byte(
    /// The integer value.
    pub i8,
);

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

/// An unsigned 8-bit integer from 0 through 255.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedByte(
    /// The integer value.
    pub u8,
);

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

/// A two's-complement signed 16-bit integer from -32,768 through 32,767.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Short(
    /// The integer value.
    pub i16,
);

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

/// An unsigned 16-bit integer from 0 through 65,535.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsignedShort(
    /// The integer value.
    pub u16,
);

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

/// A two's-complement signed 32-bit integer from -2,147,483,648 through
/// 2,147,483,647.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Int(
    /// The integer value.
    pub i32,
);

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

/// A two's-complement signed 64-bit integer from -9,223,372,036,854,775,808
/// through 9,223,372,036,854,775,807.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Long(
    /// The integer value.
    pub i64,
);

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

/// A UTF-8 string prefixed by its byte length as a VarInt.
///
/// The protocol limits both the UTF-8 payload size and the number of UTF-16
/// code units. Supplementary [Unicode scalar values] count as two UTF-16
/// code units. The general protocol limit is 32,767 UTF-16 code units and
/// three UTF-8 bytes per permitted code unit; a particular field may impose
/// a lower limit.
///
/// [Unicode scalar values]: https://www.unicode.org/glossary/#unicode_scalar_value
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrefixedString(
    /// The string value.
    pub String,
);

impl PrefixedString {
    /// The maximum number of UTF-16 code units permitted in the string.
    pub const MAX_UTF16_CODE_UNITS: usize = 0x7fff;
    /// The maximum size of the UTF-8 payload, excluding its VarInt length prefix.
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

    fn decode_value(reader: &mut impl Read) -> Result<(String, usize), CodecError> {
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
        let bytes_processed = prefix_size + byte_length;
        let value = String::from_utf8(bytes).map_err(|error| {
            let utf8_error = error.utf8_error();
            CodecError::invalid_encoding(
                CodecKind::String,
                bytes_processed,
                InvalidEncodingReason::InvalidUtf8 {
                    valid_up_to: utf8_error.valid_up_to(),
                    error_len: utf8_error.error_len(),
                },
            )
        })?;
        Self::validate_value(&value, CodecOperation::Read, bytes_processed)?;
        Ok((value, bytes_processed))
    }
}

impl TypeCodec for PrefixedString {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        Self::encode_value(&self.0, writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        Self::decode_value(reader).map(|(value, _)| Self(value))
    }
}

/// A resource identifier encoded as a [`PrefixedString`].
///
/// The namespace permits `[a-z0-9._-]`; the value permits
/// `[a-z0-9._/-]`. See the protocol's [identifier format] for details.
///
/// [identifier format]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// The maximum number of UTF-16 code units permitted in the identifier.
    pub const MAX_UTF16_CODE_UNITS: usize = PrefixedString::MAX_UTF16_CODE_UNITS;
    /// The maximum size of the UTF-8 payload, excluding its VarInt length prefix.
    pub const MAX_BYTES: usize = PrefixedString::MAX_BYTES;
    /// The maximum encoded size, including the VarInt length prefix.
    pub const MAX_ENCODED_BYTES: usize = Self::MAX_BYTES + 3;

    /// Creates an identifier after validating its namespace and path.
    ///
    /// An identifier without an explicit namespace is validated as belonging
    /// to the `minecraft` namespace, but its original spelling is preserved.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidIdentifier`] if the namespace or path is empty or
    /// contains a character not permitted by the identifier format.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the owned identifier string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TypeCodec for Identifier {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        PrefixedString::encode_value(&self.0, writer)
            .map_err(|error| error.with_context(CodecKind::Identifier))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (value, bytes_processed) = PrefixedString::decode_value(reader)
            .map_err(|error| error.with_context(CodecKind::Identifier))?;
        Self::new(value).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::Identifier,
                bytes_processed,
                InvalidEncodingReason::InvalidIdentifier,
            )
        })
    }
}

fn validate_identifier(value: &str) -> Result<(), InvalidIdentifier> {
    let (namespace, path) = match value.split_once(':') {
        Some((namespace, path)) => (namespace, path),
        None => ("minecraft", value),
    };
    let namespace_is_valid = !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        });
    let path_is_valid = !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        });
    if namespace_is_valid && path_is_valid && !path.contains(':') {
        Ok(())
    } else {
        Err(InvalidIdentifier)
    }
}

/// An error returned when a string is not a valid Minecraft resource identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIdentifier;

impl fmt::Display for InvalidIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid Minecraft identifier")
    }
}

impl std::error::Error for InvalidIdentifier {}
