pub mod basic;
pub mod compound;

use thiserror::Error;
use mcproto_codec::CodecError;

#[derive(Error, Debug)]
pub enum TypeCodecError {
    #[error("IO error occurred as type codec: {0}")]
    Io(#[from] std::io::Error),
    #[error("Empty buffer")]
    EmptyBuffer,
    #[error("Invalid boolean: {0}")]
    InvalidBoolean(u8),
    #[error("End of buffer: {0}, at least {1}")]
    EndOfBuffer(usize, usize),
    #[error("String length invalid: {0}")]
    InvalidStringLength(usize),
    #[error("Text component length invalid: {0}")]
    InvalidTextComponentLength(usize),
    #[error("Position {0} out of range: {1}")]
    InvalidPositionValue(&'static str, i32),
    #[error("Array length invalid: {0}")]
    InvalidArrayLength(i32),
    #[error("Codec error: {0}")]
    CodecError(#[from] CodecError),
    #[error("Invalid utf8")]
    InvalidUtf8,
}
pub trait Codec {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError>;
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError>
    where
        Self: Sized;
}
