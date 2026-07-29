//! Minecraft protocol types and their wire-codec interface.
//!
//! The crate models protocol values and provides their encoding and decoding
//! through [`TypeCodec`].

pub mod basic;
pub mod component;
pub mod json_text_component;
pub mod text_component;

pub trait TypeCodec {
    fn encode(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), mcproto_codec::error::CodecError>;
    fn decode(reader: &mut impl std::io::Read) -> Result<Self, mcproto_codec::error::CodecError>
    where
        Self: Sized;
}
