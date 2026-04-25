pub mod varint;
pub mod varlong;

use thiserror::Error;
pub use varint::{VarIntRead, VarIntWrite}; // 重导出
pub use varlong::{VarLongRead, VarLongWrite};
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("IO error occurred as codec: {0}")]
    Io(#[from] std::io::Error),

    #[error("VarInt too long, max is 2^31-1")]
    VarIntOverflow,

    #[error("Unexpected end of data")]
    UnexpectedEof,

    #[error("VarLong too long, max is 2^63-1")]
    VarLongOverflow,
}