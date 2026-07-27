use std::{error::Error, fmt, io};

type BoxedError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecKind {
    VarInt,
    VarLong,
    Boolean,
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    Long,
    String,
}

impl fmt::Display for CodecKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarInt => formatter.write_str("VarInt"),
            Self::VarLong => formatter.write_str("VarLong"),
            Self::Boolean => formatter.write_str("Boolean"),
            Self::Byte => formatter.write_str("Byte"),
            Self::UnsignedByte => formatter.write_str("UnsignedByte"),
            Self::Short => formatter.write_str("Short"),
            Self::UnsignedShort => formatter.write_str("UnsignedShort"),
            Self::Int => formatter.write_str("Int"),
            Self::Long => formatter.write_str("Long"),
            Self::String => formatter.write_str("String"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecOperation {
    Read,
    Write,
}

impl fmt::Display for CodecOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("reading"),
            Self::Write => formatter.write_str("writing"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InvalidEncodingReason {
    TooLong {
        max_bytes: usize,
    },
    ValueOutOfRange {
        terminal_byte: u8,
        allowed_mask: u8,
    },
    InvalidBooleanValue {
        value: u8,
    },
    StringTooLong {
        max_bytes: usize,
    },
    TooManyUtf16CodeUnits {
        max_code_units: usize,
    },
    NegativeLength {
        value: i32,
    },
    InvalidUtf8 {
        valid_up_to: usize,
        error_len: Option<usize>,
    },
}

impl fmt::Display for InvalidEncodingReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong { max_bytes } => {
                write!(formatter, "encoding exceeds the {max_bytes}-byte limit")
            }
            Self::ValueOutOfRange {
                terminal_byte,
                allowed_mask,
            } => write!(
                formatter,
                "terminal byte 0x{terminal_byte:02X} contains bits outside mask 0x{allowed_mask:02X}"
            ),
            Self::InvalidBooleanValue { value } => {
                write!(formatter, "invalid boolean value 0x{value:02X}")
            }
            Self::StringTooLong { max_bytes } => {
                write!(formatter, "string exceeds the {max_bytes}-byte UTF-8 limit")
            }
            Self::TooManyUtf16CodeUnits { max_code_units } => write!(
                formatter,
                "string exceeds the {max_code_units}-code-unit UTF-16 limit"
            ),
            Self::NegativeLength { value } => {
                write!(formatter, "length cannot be negative: {value}")
            }
            Self::InvalidUtf8 {
                valid_up_to,
                error_len: Some(error_len),
            } => write!(
                formatter,
                "invalid UTF-8 sequence of {error_len} bytes at byte {valid_up_to}"
            ),
            Self::InvalidUtf8 {
                valid_up_to,
                error_len: None,
            } => write!(
                formatter,
                "incomplete UTF-8 sequence starting at byte {valid_up_to}"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecErrorKind {
    Io,
    UnexpectedEof,
    InvalidEncoding(InvalidEncodingReason),
}

#[derive(Debug)]
pub struct CodecError {
    pub kind: CodecErrorKind,
    codec: CodecKind,
    operation: CodecOperation,
    bytes_processed: usize,
    source: Option<BoxedError>,
}

impl CodecError {
    pub const fn kind(&self) -> CodecErrorKind {
        self.kind
    }

    pub const fn codec(&self) -> CodecKind {
        self.codec
    }

    pub const fn operation(&self) -> CodecOperation {
        self.operation
    }

    pub const fn bytes_processed(&self) -> usize {
        self.bytes_processed
    }

    pub fn io_error(&self) -> Option<&io::Error> {
        self.source.as_deref()?.downcast_ref::<io::Error>()
    }

    pub fn from_read_error(codec: CodecKind, bytes_processed: usize, source: io::Error) -> Self {
        let kind = if source.kind() == io::ErrorKind::UnexpectedEof {
            CodecErrorKind::UnexpectedEof
        } else {
            CodecErrorKind::Io
        };

        Self {
            kind,
            codec,
            operation: CodecOperation::Read,
            bytes_processed,
            source: Some(Box::new(source)),
        }
    }

    pub fn from_write_error(codec: CodecKind, bytes_processed: usize, source: io::Error) -> Self {
        Self {
            kind: CodecErrorKind::Io,
            codec,
            operation: CodecOperation::Write,
            bytes_processed,
            source: Some(Box::new(source)),
        }
    }
    /// Tips：encoding是编码格式，不是encode过程
    pub const fn invalid_encoding(
        codec: CodecKind,
        bytes_processed: usize,
        reason: InvalidEncodingReason,
    ) -> Self {
        Self::invalid_encoding_for_operation(codec, CodecOperation::Read, bytes_processed, reason)
    }

    pub const fn invalid_encoding_for_operation(
        codec: CodecKind,
        operation: CodecOperation,
        bytes_processed: usize,
        reason: InvalidEncodingReason,
    ) -> Self {
        Self {
            kind: CodecErrorKind::InvalidEncoding(reason),
            codec,
            operation,
            bytes_processed,
            source: None,
        }
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CodecErrorKind::Io => write!(
                formatter,
                "I/O error while {} {} after {} bytes",
                self.operation, self.codec, self.bytes_processed
            )?,
            CodecErrorKind::UnexpectedEof => write!(
                formatter,
                "unexpected end of input while reading {} after {} bytes",
                self.codec, self.bytes_processed
            )?,
            CodecErrorKind::InvalidEncoding(reason) => write!(
                formatter,
                "invalid {} encoding after {} bytes: {reason}",
                self.codec, self.bytes_processed
            )?,
        }

        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }

        Ok(())
    }
}

impl Error for CodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}
