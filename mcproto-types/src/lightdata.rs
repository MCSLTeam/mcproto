//! Chunk-section sky and block lighting data.

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
};

use crate::{BitSet, PrefixedArray, TypeCodec};

/// Number of packed bytes in one light array.
pub const LIGHT_ARRAY_LENGTH: usize = 2048;

/// Number of four-bit light values represented by one [`LightArray`].
pub const LIGHT_VALUES_PER_ARRAY: usize = LIGHT_ARRAY_LENGTH * 2;

/// Packed light levels for one 16x16x16 chunk section.
///
/// The wire representation is a VarInt length of exactly 2048 followed by
/// 2048 bytes. Each byte stores two light levels, with the lower-indexed value
/// in the low nibble. Every light level is therefore in the range `0..=15`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LightArray(
    /// The 2048 packed light bytes.
    pub [u8; LIGHT_ARRAY_LENGTH],
);

impl LightArray {
    /// Returns the light level at a packed value index from `0` through `4095`.
    #[must_use]
    pub fn light_level(&self, index: usize) -> Option<u8> {
        if index >= LIGHT_VALUES_PER_ARRAY {
            return None;
        }
        let byte = self.0[index / 2];
        Some(if index % 2 == 0 {
            byte & 0x0f
        } else {
            byte >> 4
        })
    }

    /// Returns the packed 2048-byte payload without its length prefix.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; LIGHT_ARRAY_LENGTH] {
        &self.0
    }

    /// Extracts the packed 2048-byte payload.
    #[must_use]
    pub fn into_bytes(self) -> [u8; LIGHT_ARRAY_LENGTH] {
        self.0
    }
}

impl Default for LightArray {
    fn default() -> Self {
        Self([0; LIGHT_ARRAY_LENGTH])
    }
}

impl From<[u8; LIGHT_ARRAY_LENGTH]> for LightArray {
    fn from(bytes: [u8; LIGHT_ARRAY_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl TypeCodec for LightArray {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let prefix_size = writer
            .write_varint_with_size(LIGHT_ARRAY_LENGTH as i32)
            .map_err(|error| error.with_context(CodecKind::LightArray))?;
        write_all_counted(writer, &self.0, CodecKind::LightArray, prefix_size)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::LightArray))?;
        if length < 0 {
            return Err(CodecError::invalid_encoding(
                CodecKind::LightArray,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: length },
            ));
        }
        if length as usize != LIGHT_ARRAY_LENGTH {
            return Err(CodecError::invalid_encoding(
                CodecKind::LightArray,
                prefix_size,
                InvalidEncodingReason::ArrayLengthMismatch {
                    expected: LIGHT_ARRAY_LENGTH,
                    actual: length as usize,
                },
            ));
        }

        let mut bytes = [0; LIGHT_ARRAY_LENGTH];
        read_exact_counted(reader, &mut bytes, CodecKind::LightArray, prefix_size)?;
        Ok(Self(bytes))
    }
}

/// Lighting masks and packed light arrays for a chunk column.
///
/// Mask bit zero addresses the section immediately below the world's minimum
/// height. Subsequent bits address chunk sections from bottom to top, ending
/// with the section immediately above the world's maximum height. Sky and
/// block arrays are ordered by their corresponding set bits, least-significant
/// bit first. The empty masks identify sections whose light data is all zero.
///
/// Encoding and decoding reject a sky or block array count that differs from
/// the number of set bits in its data mask.
///
/// # Examples
///
/// ```
/// use mcproto_types::{LightData, TypeCodec};
///
/// let light = LightData::default();
/// let mut encoded = Vec::new();
/// light.encode(&mut encoded)?;
/// assert_eq!(encoded, [0, 0, 0, 0, 0, 0]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(LightData::decode(&mut input)?, light);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// See the official [Light Data] protocol documentation.
///
/// [Light Data]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Light_Data
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LightData {
    /// Sections with entries in [`sky_light_arrays`](Self::sky_light_arrays).
    pub sky_light_mask: BitSet,
    /// Sections with entries in [`block_light_arrays`](Self::block_light_arrays).
    pub block_light_mask: BitSet,
    /// Sections whose sky light data consists entirely of zeroes.
    pub empty_sky_light_mask: BitSet,
    /// Sections whose block light data consists entirely of zeroes.
    pub empty_block_light_mask: BitSet,
    /// One packed array for each set bit in [`sky_light_mask`](Self::sky_light_mask).
    pub sky_light_arrays: PrefixedArray<LightArray>,
    /// One packed array for each set bit in [`block_light_mask`](Self::block_light_mask).
    pub block_light_arrays: PrefixedArray<LightArray>,
}

impl LightData {
    /// Returns the number of sky light arrays required by the sky mask.
    #[must_use]
    pub fn expected_sky_light_array_count(&self) -> usize {
        set_bit_count(&self.sky_light_mask)
    }

    /// Returns the number of block light arrays required by the block mask.
    #[must_use]
    pub fn expected_block_light_array_count(&self) -> usize {
        set_bit_count(&self.block_light_mask)
    }

    fn validate_array_counts(&self, operation: CodecOperation) -> Result<(), CodecError> {
        validate_array_count(
            self.expected_sky_light_array_count(),
            self.sky_light_arrays.len(),
            operation,
        )?;
        validate_array_count(
            self.expected_block_light_array_count(),
            self.block_light_arrays.len(),
            operation,
        )
    }
}

impl TypeCodec for LightData {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.validate_array_counts(CodecOperation::Write)?;
        self.sky_light_mask
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))?;
        self.block_light_mask
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))?;
        self.empty_sky_light_mask
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))?;
        self.empty_block_light_mask
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))?;
        self.sky_light_arrays
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))?;
        self.block_light_arrays
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::LightData))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let value = Self {
            sky_light_mask: decode_field(reader)?,
            block_light_mask: decode_field(reader)?,
            empty_sky_light_mask: decode_field(reader)?,
            empty_block_light_mask: decode_field(reader)?,
            sky_light_arrays: decode_field(reader)?,
            block_light_arrays: decode_field(reader)?,
        };
        value.validate_array_counts(CodecOperation::Read)?;
        Ok(value)
    }
}

fn decode_field<T: TypeCodec>(reader: &mut impl Read) -> Result<T, CodecError> {
    T::decode(reader).map_err(|error| error.with_context(CodecKind::LightData))
}

fn set_bit_count(mask: &BitSet) -> usize {
    mask.0.iter().map(|word| word.count_ones() as usize).sum()
}

fn validate_array_count(
    expected: usize,
    actual: usize,
    operation: CodecOperation,
) -> Result<(), CodecError> {
    if expected == actual {
        return Ok(());
    }
    Err(CodecError::invalid_encoding_for_operation(
        CodecKind::LightData,
        operation,
        0,
        InvalidEncodingReason::ArrayLengthMismatch { expected, actual },
    ))
}
