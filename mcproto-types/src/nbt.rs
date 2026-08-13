//! Named Binary Tag (NBT) values.
//!
//! This module wraps [`fastnbt::Value`] so complete NBT payloads can be encoded
//! and decoded through [`TypeCodec`].

use std::io::{Read, Write};

use fastnbt::{DeOpts, SerOpts};
use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};

use crate::TypeCodec;

/// A Minecraft network NBT (Named Binary Tag) value.
///
/// This is a thin wrapper around [`fastnbt::Value`]. Parsing and serialization
/// are delegated directly to `fastnbt`. The wire format is network NBT, so the
/// root compound name is omitted; like `fastnbt`, the root value must be an
/// NBT compound.
///
/// # Examples
///
/// ```
/// use fastnbt::nbt;
/// use mcproto_types::{TypeCodec, nbt::Nbt};
///
/// let value = nbt!({
///     "name": "minecraft:stone",
///     "count": 1i8,
/// });
///
/// let mut encoded = Vec::new();
/// Nbt(value.clone()).encode(&mut encoded)?;
///
/// let mut input = encoded.as_slice();
/// assert_eq!(Nbt::decode(&mut input)?, Nbt(value));
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Nbt(
    /// The NBT value.
    pub fastnbt::Value,
);

impl TypeCodec for Nbt {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        fastnbt::to_writer_with_opts(writer, &self.0, SerOpts::network_nbt())
            .map_err(|source| invalid_nbt(CodecOperation::Write, source))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        fastnbt::from_reader_with_opts(reader, DeOpts::network_nbt())
            .map(Self)
            .map_err(|source| invalid_nbt(CodecOperation::Read, source))
    }
}

fn invalid_nbt(operation: CodecOperation, source: fastnbt::error::Error) -> CodecError {
    CodecError::invalid_encoding_for_operation_with_source(
        CodecKind::Nbt,
        operation,
        0,
        InvalidEncodingReason::InvalidNbt,
        source,
    )
}
