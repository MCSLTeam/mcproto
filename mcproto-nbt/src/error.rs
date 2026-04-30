use thiserror::Error;

#[derive(Debug, Error)]
pub enum NbtError {
    #[error("Unexpected end of buffer at offset {offset}: need {needed} bytes, remaining {remaining}")]
    UnexpectedEof {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    #[error("Invalid tag kind at offset {offset}: {value}")]
    InvalidTagKind {
        offset: usize,
        value: u8,
    },
    #[error("Invalid utf-8 string at offset {offset}")]
    InvalidUtf8 {
        offset: usize,
    },
    #[error("Negative length at offset {offset}: {value}")]
    NegativeLength {
        offset: usize,
        value: i32,
    },
    #[error("Depth limit exceeded")]
    DepthLimitExceeded,
}
