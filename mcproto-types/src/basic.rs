//! Basic Minecraft protocol types and their wire encodings.
//!
//! This module includes primitive numeric values, booleans, variable-length
//! integers, length-prefixed strings, and resource identifiers.

use crate::{EnumRepr, TypeCodec};
use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
    varlong::{VarLongRead, VarLongWrite},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
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

    fn encode_value(value: &str, writer: &mut impl Write) -> Result<(), CodecError> {
        encode_prefixed_string(
            value,
            writer,
            CodecKind::String,
            Self::MAX_BYTES,
            Self::MAX_UTF16_CODE_UNITS,
        )
    }

    fn decode_value(reader: &mut impl Read) -> Result<(String, usize), CodecError> {
        decode_prefixed_string(
            reader,
            CodecKind::String,
            Self::MAX_BYTES,
            Self::MAX_UTF16_CODE_UNITS,
        )
    }
}

/// Encodes `value` as a VarInt-length-prefixed string.
///
/// `codec` identifies the codec in errors, `max_bytes` limits the UTF-8
/// payload, and `max_code_units` limits the number of UTF-16 code units.
/// Supplementary Unicode scalar values count as two UTF-16 code units.
///
/// # Errors
///
/// Returns a [`CodecError`] if the string exceeds `max_bytes` or
/// `max_code_units`, or if the underlying writer fails while writing the length
/// prefix or payload.
pub(crate) fn encode_prefixed_string(
    value: &str,
    writer: &mut impl Write,
    codec: CodecKind,
    max_bytes: usize,
    max_code_units: usize,
) -> Result<(), CodecError> {
    if value.len() > max_bytes {
        return Err(CodecError::invalid_encoding_for_operation(
            codec,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::StringTooLong { max_bytes },
        ));
    }
    if value.encode_utf16().count() > max_code_units {
        return Err(CodecError::invalid_encoding_for_operation(
            codec,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::TooManyUtf16CodeUnits { max_code_units },
        ));
    }

    let bytes = value.as_bytes();
    let prefix_size = writer
        .write_varint_with_size(bytes.len() as i32)
        .map_err(|error| error.with_context(codec))?;
    write_all_counted(writer, bytes, codec, prefix_size)
}

/// Decodes a VarInt-length-prefixed string and returns its value and the total
/// number of bytes processed, including the length prefix.
///
/// `codec` is the codec in errors, `max_bytes` limits the UTF-8 payload, and
/// `max_code_units` limits the number of UTF-16 code units.
///
/// # Errors
///
/// Returns a [`CodecError`] if the length prefix is negative, the payload
/// exceeds `max_bytes`, the payload contains invalid UTF-8 or more than
/// `max_code_units` UTF-16 code units, or the reader reaches an unexpected end
/// of input.
pub(crate) fn decode_prefixed_string(
    reader: &mut impl Read,
    codec: CodecKind,
    max_bytes: usize,
    max_code_units: usize,
) -> Result<(String, usize), CodecError> {
    let (byte_length, prefix_size) = reader
        .read_varint_with_size()
        .map_err(|error| error.with_context(codec))?;
    let byte_length = usize::try_from(byte_length).map_err(|_| {
        CodecError::invalid_encoding(
            codec,
            prefix_size,
            InvalidEncodingReason::NegativeLength { value: byte_length },
        )
    })?;

    if byte_length > max_bytes {
        return Err(CodecError::invalid_encoding(
            codec,
            prefix_size,
            InvalidEncodingReason::StringTooLong { max_bytes },
        ));
    }

    let mut bytes = vec![0; byte_length];
    read_exact_counted(reader, &mut bytes, codec, prefix_size)?;
    let bytes_processed = prefix_size + byte_length;
    let value = String::from_utf8(bytes).map_err(|error| {
        let utf8_error = error.utf8_error();
        CodecError::invalid_encoding(
            codec,
            bytes_processed,
            InvalidEncodingReason::InvalidUtf8 {
                valid_up_to: utf8_error.valid_up_to(),
                error_len: utf8_error.error_len(),
            },
        )
    })?;
    if value.len() > max_bytes {
        return Err(CodecError::invalid_encoding(
            codec,
            bytes_processed,
            InvalidEncodingReason::StringTooLong { max_bytes },
        ));
    }
    if value.encode_utf16().count() > max_code_units {
        return Err(CodecError::invalid_encoding(
            codec,
            bytes_processed,
            InvalidEncodingReason::TooManyUtf16CodeUnits { max_code_units },
        ));
    }
    Ok((value, bytes_processed))
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

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
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

/// Returns whether a string is a valid Minecraft resource identifier.
///
/// An identifier without an explicit namespace is validated as belonging to
/// the `minecraft` namespace. The namespace permits `[a-z0-9._-]`; the path
/// permits `[a-z0-9._/-]`. See the protocol's [identifier format].
///
/// [identifier format]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Identifier
pub(crate) fn is_valid_identifier(value: &str) -> bool {
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
    namespace_is_valid && path_is_valid && !path.contains(':')
}

fn validate_identifier(value: &str) -> Result<(), InvalidIdentifier> {
    if is_valid_identifier(value) {
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

/// A variable-length two's-complement signed 32-bit integer.
///
/// VarInts use one to five bytes on the wire. Each byte carries seven payload
/// bits; the most-significant bit is set on every byte except the last.
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::VarInt};
///
/// let mut encoded = Vec::new();
/// VarInt(25565).encode(&mut encoded)?;
/// assert_eq!(encoded, [0xdd, 0xc7, 0x01]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(VarInt::decode(&mut input)?, VarInt(25565));
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VarInt(
    /// The integer value.
    pub i32,
);

impl TypeCodec for VarInt {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer.write_varint(self.0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        reader.read_varint().map(Self)
    }
}

/// A variable-length two's-complement signed 64-bit integer.
///
/// VarLongs use one to ten bytes on the wire. Each byte carries seven payload
/// bits; the most-significant bit is set on every byte except the last.
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::VarLong};
///
/// let mut encoded = Vec::new();
/// VarLong(9_223_372_036_854_775_000).encode(&mut encoded)?;
///
/// let mut input = encoded.as_slice();
/// assert_eq!(
///     VarLong::decode(&mut input)?,
///     VarLong(9_223_372_036_854_775_000),
/// );
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VarLong(
    /// The integer value.
    pub i64,
);

impl TypeCodec for VarLong {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        writer.write_varlong(self.0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        reader.read_varlong().map(Self)
    }
}

/// A Minecraft protocol block position packed into a 64-bit value.
///
/// The wire format stores `x` in the 26 most-significant bits, `z` in the
/// middle 26 bits, and `y` in the 12 least-significant bits. Each component is
/// a signed two's-complement integer:
///
/// ```text
/// x: 26 bits | z: 26 bits | y: 12 bits
/// ```
///
/// The value is packed as:
///
/// ```text
/// ((x & 0x3FFFFFF) << 38) | ((z & 0x3FFFFFF) << 12) | (y & 0xFFF)
/// ```
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::Position};
///
/// let position = Position {
///     x: 18_357_644,
///     y: 831,
///     z: -20_882_616,
/// };
///
/// let mut encoded = Vec::new();
/// position.encode(&mut encoded)?;
/// assert_eq!(
///     encoded,
///     [0x46, 0x07, 0x63, 0x2c, 0x15, 0xb4, 0x83, 0x3f]
/// );
///
/// let mut input = encoded.as_slice();
/// assert_eq!(Position::decode(&mut input)?, position);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// The x coordinate, from -33,554,432 through 33,554,431.
    pub x: i32,
    /// The y coordinate, from -2,048 through 2,047.
    pub y: i16,
    /// The z coordinate, from -33,554,432 through 33,554,431.
    pub z: i32,
}

impl TypeCodec for Position {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let value = ((self.x as i64 & 0x3ff_ffff) << 38)
            | ((self.z as i64 & 0x3ff_ffff) << 12)
            | (self.y as i64 & 0xfff);
        write_all_counted(writer, &value.to_be_bytes(), CodecKind::Position, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 8];
        read_exact_counted(reader, &mut bytes, CodecKind::Position, 0)?;
        let value = i64::from_be_bytes(bytes);

        Ok(Self {
            x: (value >> 38) as i32,
            y: ((value << 52) >> 52) as i16,
            z: ((value << 26) >> 38) as i32,
        })
    }
}

/// A rotation angle encoded in steps of 1/256 of a full turn.
///
/// The wire value is a single byte. Because 256 steps represent one full turn,
/// the value wraps around and its signedness does not matter.
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::Angle};
///
/// let angle = Angle(64);
///
/// let mut encoded = Vec::new();
/// angle.encode(&mut encoded)?;
/// assert_eq!(encoded, [64]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(Angle::decode(&mut input)?, angle);
///
/// assert_eq!(angle.to_degrees(), 90.0);
/// assert!((angle.to_radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Angle(
    /// The raw angle step count, from 0 through 255.
    pub u8,
);

impl Angle {
    /// Returns the angle in degrees.
    ///
    /// A full turn of 256 steps is equivalent to 360 degrees.
    pub fn to_degrees(&self) -> f64 {
        f64::from(self.0) * 360.0 / 256.0
    }

    /// Returns the angle in radians.
    ///
    /// A full turn of 256 steps is equivalent to 2π radians.
    pub fn to_radians(&self) -> f64 {
        f64::from(self.0) * std::f64::consts::TAU / 256.0
    }
}

impl TypeCodec for Angle {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &[self.0], CodecKind::Angle, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 1];
        read_exact_counted(reader, &mut bytes, CodecKind::Angle, 0)?;
        Ok(Self(bytes[0]))
    }
}

/// Three doubles compressed into a shared-scale, low-precision vector.
///
/// `LpVec3` normally occupies six bytes. The coordinates are divided by their
/// rounded-up maximum absolute value, quantized into three unsigned 15-bit
/// values, and packed with a shared scale factor:
///
/// ```text
/// X: 15 bits | Y: 15 bits | Z: 15 bits | continuation: 1 bit | scale: 2 bits
/// ```
///
/// The first two packed bytes are written in little-endian order, while the
/// remaining four bytes are written in big-endian order. If the scale factor
/// is greater than three, its upper bits follow as a VarInt. A vector whose
/// greatest absolute coordinate is below `1 / 32766`, or which contains NaN,
/// is encoded as the single byte `0x00` and decodes as zero.
///
/// This format is used by the Java Edition `Spawn Entity` and
/// `Set Entity Velocity` packets.
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::LpVec3};
///
/// let value = LpVec3::new(10.0, 0.2, -5.0);
/// let mut encoded = Vec::new();
/// value.encode(&mut encoded)?;
/// assert_eq!(encoded, [0xf6, 0xff, 0x40, 0x01, 0x05, 0x1f, 0x02]);
///
/// let mut input = encoded.as_slice();
/// let decoded = LpVec3::decode(&mut input)?;
/// assert!((decoded.x - value.x).abs() < 0.001);
/// assert!((decoded.y - value.y).abs() < 0.001);
/// assert!((decoded.z - value.z).abs() < 0.001);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LpVec3 {
    /// The x coordinate.
    pub x: f64,
    /// The y coordinate.
    pub y: f64,
    /// The z coordinate.
    pub z: f64,
}

impl LpVec3 {
    /// Greatest quantized coordinate value stored in a 15-bit field.
    pub const MAX_QUANTIZED_VALUE: f64 = 32766.0;
    /// Coordinates below this absolute maximum use the one-byte zero form.
    pub const ZERO_THRESHOLD: f64 = 1.0 / Self::MAX_QUANTIZED_VALUE;
    /// Greatest shared scale factor representable by two low bits and a
    /// 32-bit VarInt continuation.
    pub const MAX_SCALE_FACTOR: u64 = (u32::MAX as u64) << 2 | 0x03;

    const CONTINUATION_FLAG: u64 = 0x04;
    const SCALE_BITS: u64 = 0x03;

    /// Creates a low-precision vector from its coordinates.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn pack(value: f64) -> u64 {
        ((value * 0.5 + 0.5) * Self::MAX_QUANTIZED_VALUE).round() as u64
    }

    fn unpack(value: u64) -> f64 {
        ((value & 32767) as f64).min(Self::MAX_QUANTIZED_VALUE) * 2.0 / Self::MAX_QUANTIZED_VALUE
            - 1.0
    }
}

impl TypeCodec for LpVec3 {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let contains_nan = self.x.is_nan() || self.y.is_nan() || self.z.is_nan();
        let max_coordinate = self.x.abs().max(self.y.abs()).max(self.z.abs());
        if contains_nan || max_coordinate < Self::ZERO_THRESHOLD {
            return write_all_counted(writer, &[0], CodecKind::LpVec3, 0);
        }

        let scale_factor = max_coordinate.ceil() as u64;
        if scale_factor > Self::MAX_SCALE_FACTOR {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::LpVec3,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::LpVec3ScaleOutOfRange {
                    scale_factor,
                    max: Self::MAX_SCALE_FACTOR,
                },
            ));
        }

        let need_continuation = scale_factor & Self::SCALE_BITS != scale_factor;
        let packed_scale = if need_continuation {
            scale_factor & Self::SCALE_BITS | Self::CONTINUATION_FLAG
        } else {
            scale_factor
        };
        let scale = scale_factor as f64;
        let packed = Self::pack(self.x / scale) << 3
            | Self::pack(self.y / scale) << 18
            | Self::pack(self.z / scale) << 33
            | packed_scale;
        let upper = ((packed >> 16) as u32).to_be_bytes();
        let bytes = [
            packed as u8,
            (packed >> 8) as u8,
            upper[0],
            upper[1],
            upper[2],
            upper[3],
        ];
        write_all_counted(writer, &bytes, CodecKind::LpVec3, 0)?;

        if need_continuation {
            writer
                .write_varint((scale_factor >> 2) as u32 as i32)
                .map_err(|error| error.with_context(CodecKind::LpVec3))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut first = [0; 1];
        read_exact_counted(reader, &mut first, CodecKind::LpVec3, 0)?;
        if first[0] == 0 {
            return Ok(Self::default());
        }

        let mut remaining = [0; 5];
        read_exact_counted(reader, &mut remaining, CodecKind::LpVec3, 1)?;
        let upper = u32::from_be_bytes([remaining[1], remaining[2], remaining[3], remaining[4]]);
        let packed = u64::from(upper) << 16 | u64::from(remaining[0]) << 8 | u64::from(first[0]);
        let mut scale_factor = u64::from(first[0]) & Self::SCALE_BITS;
        if first[0] & Self::CONTINUATION_FLAG as u8 != 0 {
            let continuation = reader
                .read_varint()
                .map_err(|error| error.with_context(CodecKind::LpVec3))?;
            scale_factor |= u64::from(continuation as u32) << 2;
        }

        let scale = scale_factor as f64;
        Ok(Self {
            x: Self::unpack(packed >> 3) * scale,
            y: Self::unpack(packed >> 18) * scale,
            z: Self::unpack(packed >> 33) * scale,
        })
    }
}

/// A 128-bit universally unique identifier.
///
/// Encoded as an unsigned 128-bit integer (or two unsigned 64-bit integers: the
/// most significant 64 bits and then the least significant 64 bits).
///
/// See [Universally unique identifier](https://en.wikipedia.org/wiki/Universally_unique_identifier).
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::Uuid};
///
/// let value = Uuid::from_bytes([
///     0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
///     0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
/// ]);
///
/// let mut encoded = Vec::new();
/// value.encode(&mut encoded)?;
/// assert_eq!(
///     encoded,
///     [
///         0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
///         0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
///     ]
/// );
///
/// let mut input = encoded.as_slice();
/// assert_eq!(Uuid::decode(&mut input)?, value);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(
    /// The UUID value.
    pub uuid::Uuid,
);

impl Uuid {
    /// Creates a UUID from 16 bytes in big-endian order.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(uuid::Uuid::from_bytes(bytes))
    }

    /// Returns the UUID as 16 bytes in big-endian order.
    pub fn into_bytes(self) -> [u8; 16] {
        self.0.into_bytes()
    }
}

impl TypeCodec for Uuid {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        write_all_counted(writer, &self.0.into_bytes(), CodecKind::Uuid, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = [0; 16];
        read_exact_counted(reader, &mut bytes, CodecKind::Uuid, 0)?;
        Ok(Self::from_bytes(bytes))
    }
}

/// A length-prefixed bit set.
///
/// The wire representation is a VarInt length prefix followed by that many
/// 64-bit words, encoded in big-endian order. The `i`th bit is set when:
///
/// ```text
/// (data[i / 64] & (1 << (i % 64))) != 0
/// ```
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::BitSet};
///
/// let bits = BitSet(vec![0b0000_0101]);
///
/// let mut encoded = Vec::new();
/// bits.encode(&mut encoded)?;
/// assert_eq!(
///     encoded,
///     [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05]
/// );
///
/// let mut input = encoded.as_slice();
/// assert_eq!(BitSet::decode(&mut input)?, bits);
/// assert!(input.is_empty());
///
/// assert!(bits.contains(0));
/// assert!(!bits.contains(1));
/// assert!(bits.contains(2));
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BitSet(
    /// The packed 64-bit words, in little-endian bit order.
    pub Vec<u64>,
);

impl BitSet {
    /// Returns whether the bit at `index` is set.
    pub fn contains(&self, index: usize) -> bool {
        match self.0.get(index / 64) {
            Some(word) => (word & (1 << (index % 64))) != 0,
            None => false,
        }
    }
}

impl TypeCodec for BitSet {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let prefix_size = writer
            .write_varint_with_size(self.0.len() as i32)
            .map_err(|error| error.with_context(CodecKind::BitSet))?;

        let mut bytes_processed = prefix_size;
        for word in &self.0 {
            write_all_counted(
                writer,
                &word.to_be_bytes(),
                CodecKind::BitSet,
                bytes_processed,
            )?;
            bytes_processed += 8;
        }

        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::BitSet))?;
        let length = usize::try_from(length).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::BitSet,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: length },
            )
        })?;

        let mut words = Vec::with_capacity(length);
        let mut bytes_processed = prefix_size;
        for _ in 0..length {
            let mut bytes = [0; 8];
            read_exact_counted(reader, &mut bytes, CodecKind::BitSet, bytes_processed)?;
            words.push(u64::from_be_bytes(bytes));
            bytes_processed += 8;
        }

        Ok(Self(words))
    }
}

/// A bit set with a fixed length of `N` bits.
///
/// A fixed bit set is encoded as exactly `ceil(N / 8)` bytes, without a
/// length prefix. This differs from [`BitSet`], which is prefixed by a VarInt
/// and stores packed 64-bit words. The packed bytes follow the same bit order
/// as Java's `BitSet.toByteArray`: bit `i` is set when the following expression
/// is non-zero:
///
/// ```text
/// (data[i / 8] & (1 << (i % 8))) != 0
/// ```
///
/// The final byte is padded with zero bits when `N` is not divisible by eight.
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::FixedBitSet};
///
/// // Bits 0, 7, and 8 are set in a nine-bit set.
/// let bits = FixedBitSet::<9>(vec![0b1000_0001, 0b0000_0001]);
///
/// let mut encoded = Vec::new();
/// bits.encode(&mut encoded)?;
/// assert_eq!(encoded, [0b1000_0001, 0b0000_0001]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(FixedBitSet::<9>::decode(&mut input)?, bits);
/// assert!(bits.contains(0));
/// assert!(bits.contains(7));
/// assert!(bits.contains(8));
/// assert!(!bits.contains(9));
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct FixedBitSet<const N: usize>(
    /// The packed bytes, in little-endian bit order within each byte.
    pub Vec<u8>,
);

impl<const N: usize> FixedBitSet<N> {
    /// The number of packed bytes used by this fixed bit set.
    pub const BYTE_LEN: usize = N.div_ceil(8);

    /// Returns whether the bit at `index` is set.
    ///
    /// Indices outside this set's fixed length return `false`.
    pub fn contains(&self, index: usize) -> bool {
        index < N
            && self
                .0
                .get(index / 8)
                .is_some_and(|byte| (byte & (1 << (index % 8))) != 0)
    }

    fn validate(
        &self,
        operation: CodecOperation,
        bytes_processed: usize,
    ) -> Result<(), CodecError> {
        if self.0.len() != Self::BYTE_LEN {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::FixedBitSet,
                operation,
                bytes_processed,
                InvalidEncodingReason::InvalidFixedBitSetLength {
                    expected: Self::BYTE_LEN,
                    actual: self.0.len(),
                },
            ));
        }

        if let Some(last_byte) = self.0.last()
            && N % 8 != 0
        {
            let allowed_mask = (1u8 << (N % 8)) - 1;
            if last_byte & !allowed_mask != 0 {
                return Err(CodecError::invalid_encoding_for_operation(
                    CodecKind::FixedBitSet,
                    operation,
                    bytes_processed,
                    InvalidEncodingReason::ValueOutOfRange {
                        terminal_byte: *last_byte,
                        allowed_mask,
                    },
                ));
            }
        }

        Ok(())
    }
}

impl<const N: usize> TypeCodec for FixedBitSet<N> {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.validate(CodecOperation::Write, 0)?;
        write_all_counted(writer, &self.0, CodecKind::FixedBitSet, 0)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut bytes = vec![0; Self::BYTE_LEN];
        read_exact_counted(reader, &mut bytes, CodecKind::FixedBitSet, 0)?;

        let value = Self(bytes);
        value.validate(CodecOperation::Read, Self::BYTE_LEN)?;
        Ok(value)
    }
}

macro_rules! impl_enum_repr {
    ($type:ident, $primitive:ty) => {
        impl EnumRepr for $type {
            fn from_discriminant(value: i128) -> Option<Self> {
                <$primitive>::try_from(value).ok().map(Self)
            }

            fn discriminant(&self) -> i128 {
                self.0 as i128
            }
        }
    };
}

impl EnumRepr for Boolean {
    fn from_discriminant(value: i128) -> Option<Self> {
        match value {
            0 => Some(Self(false)),
            1 => Some(Self(true)),
            _ => None,
        }
    }

    fn discriminant(&self) -> i128 {
        if self.0 { 1 } else { 0 }
    }
}

impl_enum_repr!(Byte, i8);
impl_enum_repr!(UnsignedByte, u8);
impl_enum_repr!(Short, i16);
impl_enum_repr!(UnsignedShort, u16);
impl_enum_repr!(Int, i32);
impl_enum_repr!(Long, i64);
impl_enum_repr!(VarInt, i32);
impl_enum_repr!(VarLong, i64);
impl_enum_repr!(Angle, u8);
