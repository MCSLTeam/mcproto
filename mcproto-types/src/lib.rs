//! Minecraft protocol types and their wire-codec interface.
//!
//! The crate models protocol values and provides their encoding and decoding
//! through [`TypeCodec`] and [`ContextualCodec`].

pub mod basic;
pub mod component;
pub mod contextual;
pub mod json_text_component;
pub mod nbt;
pub mod text_component;

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
