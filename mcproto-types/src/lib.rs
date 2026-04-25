pub mod basic;

use thiserror::Error;

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
}
pub trait Codec {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError>;
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError>
    where
        Self: Sized;
}