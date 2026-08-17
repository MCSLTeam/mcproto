//! Minecraft protocol types and their wire-codec interface.
//!
//! The crate models protocol values and provides their encoding and decoding
//! through [`TypeCodec`] and [`ContextualCodec`].

extern crate self as mcproto_types;

pub mod basic;
pub mod chat_type;
pub mod component;
pub mod contextual;
pub mod debug;
pub mod entity_metadata;
pub mod json_text_component;
pub mod lightdata;
pub mod nbt;
pub mod profile;
pub mod recipe;
pub mod slot;
pub mod sound_event;
pub mod teleport_flags;
pub mod text_component;

/// Re-exports all protocol types at the crate root.
///
/// The original module paths remain available, so both
/// `mcproto_types::basic::VarInt` and `mcproto_types::VarInt` are supported.
pub use basic::*;
pub use chat_type::*;
pub use component::*;
pub use contextual::*;
pub use debug::*;
pub use entity_metadata::*;
pub use json_text_component::*;
pub use lightdata::*;
pub use nbt::*;
pub use profile::*;
pub use recipe::*;
pub use slot::*;
pub use sound_event::*;
pub use teleport_flags::*;
pub use text_component::*;

/// Re-exports of protocol codec derive macros.
///
/// [`ProtocolEnum`] implements both [`ProtocolEnum`](trait@ProtocolEnum) and
/// [`TypeCodec`] for a fieldless enum. [`TypeStructCodec`] implements
/// [`TypeCodec`] for a structure by processing its fields in declaration order.
pub use mcproto_derive::{ProtocolEnum, TypeStructCodec};

/// Encodes and decodes a value whose wire representation depends on external
/// protocol context.
///
/// Unlike [`TypeCodec`], this trait receives a [`Context`](contextual::Context)
/// supplied by the enclosing packet or data structure. The context itself is
/// not written to or read from the wire. For an optional field, the caller must
/// determine whether the field is present from the surrounding protocol data.
///
/// # Examples
///
/// A generic helper can encode any contextual value using context supplied by
/// its enclosing packet:
///
/// ```
/// use mcproto_codec::error::CodecError;
/// use mcproto_types::{ContextualCodec, contextual::Context};
///
/// fn encode_contextual<T: ContextualCodec>(
///     value: &T,
///     context: &Context,
/// ) -> Result<Vec<u8>, CodecError> {
///     let mut encoded = Vec::new();
///     value.encode_with_context(&mut encoded, context)?;
///     Ok(encoded)
/// }
/// ```
pub trait ContextualCodec {
    /// Encodes this value using context supplied by its enclosing structure.
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &contextual::Context,
    ) -> Result<(), mcproto_codec::error::CodecError>;

    /// Decodes this value using context supplied by its enclosing structure.
    fn decode_with_context(
        reader: &mut impl std::io::Read,
        context: &contextual::Context,
    ) -> Result<Self, mcproto_codec::error::CodecError>
    where
        Self: Sized;
}

/// Encodes and decodes a value with a context-independent wire representation.
pub trait TypeCodec {
    /// Encodes this value to the writer.
    fn encode(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), mcproto_codec::error::CodecError>;

    /// Decodes this value from the reader.
    fn decode(reader: &mut impl std::io::Read) -> Result<Self, mcproto_codec::error::CodecError>
    where
        Self: Sized;
}

/// A numeric protocol type that can represent an enum discriminant.
///
/// This trait is implemented by the numeric types in [`basic`]. It is used by
/// [`ProtocolEnum`] to map an enum's discriminants to its wire representation.
/// Custom numeric protocol types may implement this trait to support them in
/// `#[derive(ProtocolEnum)]`.
pub trait EnumRepr: TypeCodec {
    /// Converts a Rust enum discriminant into this wire representation.
    #[must_use]
    fn from_discriminant(value: i128) -> Option<Self>
    where
        Self: Sized;

    /// Returns this representation as a numeric enum discriminant.
    #[must_use]
    fn discriminant(&self) -> i128;
}

/// Maps a fieldless Rust enum to a numeric Minecraft protocol representation.
///
/// Implement this trait with [`ProtocolEnum`] derive:
///
/// ```
/// use mcproto_types::{ProtocolEnum, TypeCodec, basic::VarInt};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
/// #[protocol_enum(repr = VarInt)]
/// enum GameMode {
///     Survival = 0,
///     Creative = 1,
/// }
///
/// let mut encoded = Vec::new();
/// GameMode::Creative.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x01]);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
pub trait ProtocolEnum: Sized {
    /// The numeric protocol type used for this enum on the wire.
    type Repr: EnumRepr;

    /// Returns this variant's numeric discriminant.
    #[must_use]
    fn discriminant(&self) -> i128;

    /// Converts this variant to its wire representation.
    ///
    /// Returns [`None`] when the discriminant does not fit in [`Self::Repr`].
    #[must_use]
    fn to_repr(&self) -> Option<Self::Repr>;

    /// Maps a decoded wire representation to a declared enum variant.
    ///
    /// Returns [`None`] for a value that does not name a declared variant.
    #[must_use]
    fn from_repr(repr: Self::Repr) -> Option<Self>;
}

/// Implementation details used by derive macros.
///
/// This module is public only so [`ProtocolEnum`] can be expanded in crates
/// that depend on `mcproto-types`; it is not part of the stable public API.
#[doc(hidden)]
pub mod __private {
    use std::io::{self, Read, Write};

    pub use mcproto_codec::error::{CodecError, CodecKind};

    use super::{EnumRepr, ProtocolEnum, TypeCodec};
    use mcproto_codec::error::{CodecOperation, InvalidEncodingReason};

    /// Encodes a value generated by [`ProtocolEnum`] derive.
    pub fn encode_protocol_enum<E: ProtocolEnum>(
        value: &E,
        writer: &mut impl Write,
    ) -> Result<(), CodecError> {
        let repr = value.to_repr().ok_or_else(|| {
            CodecError::invalid_encoding_for_operation(
                CodecKind::Enum,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::EnumDiscriminantOutOfRange {
                    value: value.discriminant(),
                },
            )
        })?;
        repr.encode(writer)
            .map_err(|error| error.with_context(CodecKind::Enum))
    }

    /// Decodes a value generated by [`ProtocolEnum`] derive.
    pub fn decode_protocol_enum<E: ProtocolEnum>(reader: &mut impl Read) -> Result<E, CodecError> {
        let mut reader = CountingReader {
            reader,
            bytes_processed: 0,
        };
        let repr =
            E::Repr::decode(&mut reader).map_err(|error| error.with_context(CodecKind::Enum))?;
        let value = repr.discriminant();

        E::from_repr(repr).ok_or_else(|| {
            CodecError::invalid_encoding(
                CodecKind::Enum,
                reader.bytes_processed,
                InvalidEncodingReason::InvalidEnumValue { value },
            )
        })
    }

    struct CountingReader<'a, R> {
        reader: &'a mut R,
        bytes_processed: usize,
    }

    impl<R: Read> Read for CountingReader<'_, R> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let read = self.reader.read(buffer)?;
            self.bytes_processed += read;
            Ok(read)
        }
    }
}

/// Adapts a context-independent [`TypeCodec`] to [`ContextualCodec`].
///
/// The supplied context is ignored because the value's wire representation is
/// already complete without external information. Context-sensitive types
/// should implement [`ContextualCodec`] directly instead of [`TypeCodec`].
impl<T> ContextualCodec for T
where
    T: TypeCodec,
{
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        _context: &contextual::Context,
    ) -> Result<(), mcproto_codec::error::CodecError> {
        self.encode(writer)
    }

    fn decode_with_context(
        reader: &mut impl std::io::Read,
        _context: &contextual::Context,
    ) -> Result<Self, mcproto_codec::error::CodecError>
    where
        Self: Sized,
    {
        Self::decode(reader)
    }
}

/// Encodes a boxed protocol value using the value's own codec.
impl<T> TypeCodec for Box<T>
where
    T: TypeCodec,
{
    fn encode(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), mcproto_codec::error::CodecError> {
        self.as_ref().encode(writer)
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, mcproto_codec::error::CodecError> {
        T::decode(reader).map(Self::new)
    }
}
