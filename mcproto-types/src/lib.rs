pub mod basic;
pub mod compound;
pub mod contextual;

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
    #[error("Missing context field: {0}")]
    MissingContext(&'static str),
    #[error("Context mismatch: {0}")]
    ContextMismatch(&'static str),
    #[error("Invalid ID-or-X value: {0}")]
    InvalidIdOrValue(i32),
    #[error("Codec error: {0}")]
    CodecError(#[from] CodecError),
    #[error("Invalid utf8")]
    InvalidUtf8,
    #[error("NBT tag unknown: {0}")]
    UnknownNbtTag(u8),
}
pub trait Codec {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError>;
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError>
    where
        Self: Sized;
}

pub struct Ctx {
    pub present: Option<bool>,      // Optional
    pub len: Option<usize>,         // Array / fixed bytes
    pub bits: Option<usize>,        // FixedBitSet(n)
    pub max_chars: Option<usize>,   // String/Text limit
    pub tag: Option<i32>,           // union/enum selector
}


pub trait ContextualCodec<Ctx = ()>: Sized {
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, ctx: &Ctx) -> Result<(), TypeCodecError>;
    fn decode_with_ctx(buf: &mut &[u8], ctx: &Ctx) -> Result<Self, TypeCodecError>;
}

impl<T> ContextualCodec<()> for T
where
    T: Codec,
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, _ctx: &()) -> Result<(), TypeCodecError> {
        self.encode(buf)
    }

    fn decode_with_ctx(buf: &mut &[u8], _ctx: &()) -> Result<Self, TypeCodecError> {
        Self::decode(buf)
    }
}
