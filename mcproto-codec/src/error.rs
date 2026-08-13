//! Errors reported by Minecraft protocol codecs.
//!
//! This module provides structured context for failures while reading and
//! writing the protocol values implemented by `mcproto-codec`.

use std::{error::Error, fmt, io};

type BoxedError = Box<dyn Error + Send + Sync + 'static>;

/// Identifies the protocol codec that reported an error.
///
/// A [`CodecError`] stores the codec that originally reported the error and may
/// also store enclosing codecs as additional context. Protocol descriptions are
/// based on the [Minecraft Java Edition protocol packet format].
///
/// Signed integer codecs use [two's-complement] representation.
///
/// [Minecraft Java Edition protocol packet format]: https://minecraft.wiki/w/Java_Edition_protocol/Packets
/// [two's-complement]: https://en.wikipedia.org/wiki/Two%27s_complement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecKind {
    /// A variable-length, two's-complement signed 32-bit integer.
    ///
    /// Values range from -2,147,483,648 through 2,147,483,647.
    VarInt,
    /// A variable-length, two's-complement signed 64-bit integer.
    ///
    /// Values range from -9,223,372,036,854,775,808 through
    /// 9,223,372,036,854,775,807.
    VarLong,
    /// A complete Named Binary Tag value.
    ///
    /// The value is encoded and decoded using `fastnbt`.
    Nbt,
    /// A boolean encoded as `0x00` for false or `0x01` for true.
    Boolean,
    /// A two's-complement signed 8-bit integer from -128 through 127.
    Byte,
    /// An unsigned 8-bit integer from 0 through 255.
    UnsignedByte,
    /// A two's-complement signed 16-bit integer from -32,768 through 32,767.
    Short,
    /// An unsigned 16-bit integer from 0 through 65,535.
    UnsignedShort,
    /// A two's-complement signed 32-bit integer from -2,147,483,648 through
    /// 2,147,483,647.
    Int,
    /// A two's-complement signed 64-bit integer from -9,223,372,036,854,775,808
    /// through 9,223,372,036,854,775,807.
    Long,
    /// A block position packed into a 64-bit integer.
    ///
    /// The x, z, and y coordinates occupy 26, 26, and 12 bits respectively.
    Position,
    /// A rotation angle encoded in 1/256 turn steps.
    Angle,
    /// A 128-bit universally unique identifier.
    Uuid,
    /// A length-prefixed bit set of packed 64-bit words.
    BitSet,
    /// A fixed-length bit set of packed bytes.
    FixedBitSet,
    /// A value whose presence is determined by an enclosing protocol context.
    Optional,
    /// An optional value prefixed by an encoded boolean presence marker.
    PrefixedOptional,
    /// A UTF-8 string prefixed by its byte length as a VarInt.
    ///
    /// The protocol limits both the UTF-8 payload size and the number of UTF-16
    /// code units. Supplementary [Unicode scalar values] count as two UTF-16
    /// code units. The general protocol limit is 32,767 UTF-16 code units and
    /// three UTF-8 bytes per permitted code unit; a particular field may impose
    /// a lower limit.
    ///
    /// [Unicode scalar values]: https://www.unicode.org/glossary/#unicode_scalar_value
    String,
    /// A resource identifier encoded as a [`String`](Self::String).
    ///
    /// The namespace permits `[a-z0-9._-]`; the value permits
    /// `[a-z0-9._/-]`. See the protocol's [identifier format] for details.
    ///
    /// [identifier format]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Identifier
    Identifier,
    /// A text component encoded as an NBT tag.
    ///
    /// Plain text-only components may use an NBT string tag. Components with
    /// styling, events, or other data use an NBT compound tag. See the
    /// [text component format] and [NBT specification].
    ///
    /// [text component format]: https://minecraft.wiki/w/Text_component_format
    /// [NBT specification]: https://minecraft.wiki/w/NBT_format
    TextComponent,
    /// A text component encoded as JSON in a protocol string.
    ///
    /// Since Java Edition 1.20.3, the vanilla implementation permits up to
    /// 262,144 UTF-16 code units when decoding but refuses to encode more than
    /// 32,767. See the [text component format].
    ///
    /// [text component format]: https://minecraft.wiki/w/Text_component_format
    JsonTextComponent,
}

/// Formats a codec kind using its protocol name, such as `VarInt` or `Boolean`.
impl fmt::Display for CodecKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarInt => formatter.write_str("VarInt"),
            Self::VarLong => formatter.write_str("VarLong"),
            Self::Nbt => formatter.write_str("Nbt"),
            Self::Boolean => formatter.write_str("Boolean"),
            Self::Byte => formatter.write_str("Byte"),
            Self::UnsignedByte => formatter.write_str("UnsignedByte"),
            Self::Short => formatter.write_str("Short"),
            Self::UnsignedShort => formatter.write_str("UnsignedShort"),
            Self::Int => formatter.write_str("Int"),
            Self::Long => formatter.write_str("Long"),
            Self::Position => formatter.write_str("Position"),
            Self::Angle => formatter.write_str("Angle"),
            Self::Uuid => formatter.write_str("UUID"),
            Self::BitSet => formatter.write_str("BitSet"),
            Self::FixedBitSet => formatter.write_str("Fixed BitSet"),
            Self::Optional => formatter.write_str("Optional"),
            Self::PrefixedOptional => formatter.write_str("Prefixed Optional"),
            Self::String => formatter.write_str("String"),
            Self::Identifier => formatter.write_str("Identifier"),
            Self::TextComponent => formatter.write_str("TextComponent"),
            Self::JsonTextComponent => formatter.write_str("JsonTextComponent"),
        }
    }
}

/// Identifies whether an error occurred while decoding or encoding data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecOperation {
    /// A read (decoding) operation.
    Read,
    /// A write (encoding) operation.
    Write,
}

/// Formats an operation as `reading` or `writing`.
impl fmt::Display for CodecOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => formatter.write_str("reading"),
            Self::Write => formatter.write_str("writing"),
        }
    }
}
/// Describes why encoded protocol data is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InvalidEncodingReason {
    /// The encoding exceeds the maximum allowed length in bytes.
    TooLong {
        /// The maximum number of bytes permitted for this encoding.
        max_bytes: usize,
    },
    /// The terminal byte of the encoding contains bits outside the allowed mask.
    ValueOutOfRange {
        /// The final byte that contains disallowed bits.
        terminal_byte: u8,
        /// A mask whose set bits identify the permitted bits in the final byte.
        allowed_mask: u8,
    },
    /// The boolean value is invalid (not 0x00 or 0x01).
    InvalidBooleanValue {
        /// The byte read instead of the permitted `0x00` or `0x01`.
        value: u8,
    },
    /// The string exceeds the maximum allowed length in bytes when encoded in UTF-8.
    StringTooLong {
        /// The maximum permitted size of the UTF-8 payload, excluding its
        /// VarInt length prefix.
        max_bytes: usize,
    },
    /// The string exceeds the maximum allowed length in UTF-16 code units.
    TooManyUtf16CodeUnits {
        /// The maximum permitted number of UTF-16 code units.
        max_code_units: usize,
    },
    /// The length of the data is negative, which is invalid.
    NegativeLength {
        /// The negative length decoded from the data.
        value: i32,
    },
    /// The packed byte array does not have the required fixed length.
    InvalidFixedBitSetLength {
        /// The expected number of packed bytes.
        expected: usize,
        /// The actual number of packed bytes.
        actual: usize,
    },
    /// An optional value does not agree with its externally supplied context.
    OptionalValueMismatch {
        /// Whether the context says that the value is present on the wire.
        context_present: bool,
        /// Whether the value held by the wrapper is present in memory.
        value_present: bool,
    },
    /// The data contains an invalid UTF-8 sequence.
    InvalidUtf8 {
        /// The byte offset in the UTF-8 payload up to which the data is valid.
        valid_up_to: usize,
        /// The length of the invalid sequence, or `None` if the input ends in
        /// an incomplete sequence.
        error_len: Option<usize>,
    },
    /// The data is not a valid Minecraft identifier.
    InvalidIdentifier,
    /// The data is not valid NBT (Named Binary Tag) data.
    InvalidNbt,
    /// The data is not valid JSON.
    InvalidJson,
    /// The root tag of a text component is invalid (not TAG_String or TAG_Compound).
    InvalidTextComponentRootTag {
        /// The unsupported NBT root tag identifier.
        tag: u8,
    },
}
/// Formats an invalid encoding reason as a diagnostic message.
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
            Self::InvalidFixedBitSetLength { expected, actual } => write!(
                formatter,
                "fixed bit set requires {expected} packed bytes, got {actual}"
            ),
            Self::OptionalValueMismatch {
                context_present,
                value_present,
            } => write!(
                formatter,
                "optional value presence ({value_present}) does not match context ({context_present})"
            ),
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
            Self::InvalidIdentifier => formatter.write_str("invalid Minecraft identifier"),
            Self::InvalidNbt => formatter.write_str("invalid NBT data"),
            Self::InvalidJson => formatter.write_str("invalid JSON data"),
            Self::InvalidTextComponentRootTag { tag } => write!(
                formatter,
                "text component root tag must be TAG_String (8) or TAG_Compound (10), got {tag}"
            ),
        }
    }
}
/// Classifies an error reported by a protocol codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecErrorKind {
    /// An I/O error other than an unexpected end of input occurred.
    Io,
    /// A read ended before the codec received all required bytes.
    UnexpectedEof,
    /// The data could not be decoded or encoded according to the codec's
    /// format or limits.
    InvalidEncoding(InvalidEncodingReason),
}

/// An error produced while reading or writing protocol data.
///
/// The error records the originating [`CodecKind`], the [`CodecOperation`], the
/// progress within that codec, and optional enclosing codec contexts. I/O and
/// parser errors are retained as an error [`source`](Error::source).
///
/// Error enums are non-exhaustive, so downstream matches must include a
/// wildcard arm.
///
/// # Example
///
/// ```
/// use mcproto_codec::{
///     error::{CodecErrorKind, CodecKind, CodecOperation},
///     varint::VarIntRead,
/// };
///
/// let mut input = [0x80].as_slice();
/// let error = input
///     .read_varint()
///     .unwrap_err()
///     .with_context(CodecKind::String);
///
/// assert_eq!(error.codec(), CodecKind::VarInt);
/// assert_eq!(error.operation(), CodecOperation::Read);
/// assert_eq!(error.bytes_processed(), 1);
/// assert_eq!(error.contexts(), &[CodecKind::String]);
///
/// match error.kind() {
///     CodecErrorKind::UnexpectedEof => {}
///     _ => panic!("unexpected error: {error}"),
/// }
/// ```
#[derive(Debug)]
pub struct CodecError {
    /// The error classification.
    ///
    /// This field and [`kind`](Self::kind) expose the same value. The accessor
    /// is convenient when working through a shared reference.
    pub kind: CodecErrorKind,
    codec: CodecKind,
    contexts: Contexts,
    operation: CodecOperation,
    bytes_processed: usize,
    source: Option<BoxedError>,
}

/// Stores the enclosing codec contexts of a [`CodecError`].
///
/// The common cases of zero or one context are stored without heap allocation;
/// only longer chains fall back to a [`Vec`].
#[derive(Debug, Default)]
enum Contexts {
    /// No enclosing contexts.
    #[default]
    None,
    /// A single context, stored inline.
    One(CodecKind),
    /// Two or more contexts, stored in a heap-allocated vector.
    Many(Vec<CodecKind>),
}

impl CodecError {
    /// Returns the error classification.
    pub const fn kind(&self) -> CodecErrorKind {
        self.kind
    }
    /// Returns the codec that originally reported the error.
    pub const fn codec(&self) -> CodecKind {
        self.codec
    }
    /// Returns the outermost enclosing codec context, if one was added.
    ///
    /// This is the last element of [`contexts`](Self::contexts), not the
    /// originating codec returned by [`codec`](Self::codec).
    pub fn context(&self) -> Option<CodecKind> {
        self.contexts().last().copied()
    }
    /// Returns all enclosing codec contexts, ordered from nearest to outermost.
    ///
    /// The originating codec is not included. Each call to
    /// [`with_context`](Self::with_context) appends one element.
    pub fn contexts(&self) -> &[CodecKind] {
        match &self.contexts {
            Contexts::None => &[],
            Contexts::One(context) => std::slice::from_ref(context),
            Contexts::Many(contexts) => contexts,
        }
    }
    /// Returns the operation being performed when the error occurred.
    pub const fn operation(&self) -> CodecOperation {
        self.operation
    }
    /// Returns the byte progress reported by the originating codec.
    ///
    /// Built-in codecs count bytes from the start of their encoded value. Bytes
    /// successfully read or written before an I/O failure are included. A byte
    /// that was read and then found to be invalid is also included. For a
    /// length-prefixed value, the originating codec determines whether its
    /// prefix is part of the count.
    ///
    /// Adding an outer context does not translate this value into an offset
    /// within the enclosing codec.
    pub const fn bytes_processed(&self) -> usize {
        self.bytes_processed
    }
    /// Returns the underlying [`io::Error`], if the source is an I/O error.
    ///
    /// Invalid NBT or JSON errors may have a non-I/O source; access those
    /// through [`Error::source`] instead.
    pub fn io_error(&self) -> Option<&io::Error> {
        self.source.as_deref()?.downcast_ref::<io::Error>()
    }

    /// Adds an enclosing codec to the error's context chain.
    ///
    /// Contexts should be added as the error propagates outward. Repeated calls
    /// therefore order [`contexts`](Self::contexts) from nearest to outermost,
    /// and [`context`](Self::context) returns the most recently added context.
    pub fn with_context(mut self, context: CodecKind) -> Self {
        self.contexts = match self.contexts {
            Contexts::None => Contexts::One(context),
            Contexts::One(first) => Contexts::Many(vec![first, context]),
            Contexts::Many(mut contexts) => {
                contexts.push(context);
                Contexts::Many(contexts)
            }
        };
        self
    }
    /// Creates an error from an I/O failure that occurred while reading.
    ///
    /// [`io::ErrorKind::UnexpectedEof`] maps to
    /// [`CodecErrorKind::UnexpectedEof`]; every other error kind maps to
    /// [`CodecErrorKind::Io`]. The source error is retained.
    ///
    /// `bytes_processed` is the number of bytes read before `source` occurred.
    pub fn from_read_error(codec: CodecKind, bytes_processed: usize, source: io::Error) -> Self {
        let kind = if source.kind() == io::ErrorKind::UnexpectedEof {
            CodecErrorKind::UnexpectedEof
        } else {
            CodecErrorKind::Io
        };

        Self {
            kind,
            codec,
            contexts: Contexts::None,
            operation: CodecOperation::Read,
            bytes_processed,
            source: Some(Box::new(source)),
        }
    }
    /// Creates an error from an I/O failure that occurred while writing.
    ///
    /// All write errors map to [`CodecErrorKind::Io`], and the source error is
    /// retained. `bytes_processed` is the number of bytes written before
    /// `source` occurred.
    pub fn from_write_error(codec: CodecKind, bytes_processed: usize, source: io::Error) -> Self {
        Self {
            kind: CodecErrorKind::Io,
            codec,
            contexts: Contexts::None,
            operation: CodecOperation::Write,
            bytes_processed,
            source: Some(Box::new(source)),
        }
    }
    /// Creates an invalid encoding error for a read operation.
    ///
    /// Use [`invalid_encoding_for_operation`](Self::invalid_encoding_for_operation)
    /// when the operation is not necessarily [`CodecOperation::Read`].
    pub const fn invalid_encoding(
        codec: CodecKind,
        bytes_processed: usize,
        reason: InvalidEncodingReason,
    ) -> Self {
        Self::invalid_encoding_for_operation(codec, CodecOperation::Read, bytes_processed, reason)
    }

    /// Creates an invalid encoding error for the specified operation.
    ///
    /// Unlike [`invalid_encoding`](Self::invalid_encoding), this constructor
    /// does not assume that the error occurred while reading.
    pub const fn invalid_encoding_for_operation(
        codec: CodecKind,
        operation: CodecOperation,
        bytes_processed: usize,
        reason: InvalidEncodingReason,
    ) -> Self {
        Self {
            kind: CodecErrorKind::InvalidEncoding(reason),
            codec,
            contexts: Contexts::None,
            operation,
            bytes_processed,
            source: None,
        }
    }
    /// Creates an invalid encoding error with an underlying source error.
    ///
    /// `operation` may be either reading or writing. The supplied error is
    /// available through [`Error::source`]; if it is an [`io::Error`], it is
    /// also available through [`io_error`](Self::io_error).
    pub fn invalid_encoding_for_operation_with_source(
        codec: CodecKind,
        operation: CodecOperation,
        bytes_processed: usize,
        reason: InvalidEncodingReason,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind: CodecErrorKind::InvalidEncoding(reason),
            codec,
            contexts: Contexts::None,
            operation,
            bytes_processed,
            source: Some(Box::new(source)),
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

        for context in self.contexts() {
            write!(formatter, " while processing {context}")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn read_error() -> CodecError {
        CodecError::from_read_error(
            CodecKind::VarInt,
            3,
            io::Error::new(io::ErrorKind::UnexpectedEof, "stream ended"),
        )
    }

    fn write_error() -> CodecError {
        CodecError::from_write_error(CodecKind::String, 5, io::Error::other("disk full"))
    }

    fn invalid_encoding_error() -> CodecError {
        CodecError::invalid_encoding_for_operation(
            CodecKind::Boolean,
            CodecOperation::Read,
            1,
            InvalidEncodingReason::InvalidBooleanValue { value: 2 },
        )
    }

    fn invalid_encoding_with_source() -> CodecError {
        CodecError::invalid_encoding_for_operation_with_source(
            CodecKind::JsonTextComponent,
            CodecOperation::Read,
            4,
            InvalidEncodingReason::InvalidJson,
            io::Error::new(io::ErrorKind::InvalidData, "bad json"),
        )
    }

    #[test]
    fn display_reports_unexpected_eof_operation_and_progress() {
        assert_eq!(
            read_error().to_string(),
            "unexpected end of input while reading VarInt after 3 bytes: stream ended"
        );
    }

    #[test]
    fn display_reports_write_io_errors() {
        assert_eq!(
            write_error().to_string(),
            "I/O error while writing String after 5 bytes: disk full"
        );
    }

    #[test]
    fn display_reports_invalid_encoding_reason() {
        assert_eq!(
            invalid_encoding_error().to_string(),
            "invalid Boolean encoding after 1 bytes: invalid boolean value 0x02"
        );
    }

    #[test]
    fn display_appends_contexts_and_source_in_order() {
        let error = invalid_encoding_with_source()
            .with_context(CodecKind::String)
            .with_context(CodecKind::Identifier)
            .with_context(CodecKind::TextComponent);
        assert_eq!(
            error.to_string(),
            "invalid JsonTextComponent encoding after 4 bytes: invalid JSON data \
             while processing String while processing Identifier while processing TextComponent: bad json"
        );
    }

    #[test]
    fn display_omits_contexts_and_source_when_absent() {
        let error = invalid_encoding_error();
        assert!(!error.to_string().contains("while processing"));
        assert!(
            !error.to_string().ends_with(": invalid boolean value 0x02:"),
            "a source was rendered when none is stored"
        );
    }

    #[test]
    fn contexts_are_empty_by_default() {
        let error = read_error();
        assert!(error.contexts().is_empty());
        assert_eq!(error.context(), None);
    }

    #[test]
    fn single_context_is_reported_inline() {
        let error = read_error().with_context(CodecKind::String);
        assert_eq!(error.contexts(), &[CodecKind::String]);
        assert_eq!(error.context(), Some(CodecKind::String));
    }

    #[test]
    fn many_contexts_are_reported_nearest_to_outermost() {
        let error = invalid_encoding_error()
            .with_context(CodecKind::String)
            .with_context(CodecKind::Identifier)
            .with_context(CodecKind::TextComponent);
        assert_eq!(
            error.contexts(),
            &[
                CodecKind::String,
                CodecKind::Identifier,
                CodecKind::TextComponent
            ]
        );
        assert_eq!(error.context(), Some(CodecKind::TextComponent));
        assert_eq!(error.codec(), CodecKind::Boolean);
    }

    #[test]
    fn io_error_returns_the_underlying_io_error() {
        let error = read_error();
        let io_error = error.io_error().expect("io_error() should be Some");
        assert_eq!(io_error.kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(io_error.to_string(), "stream ended");
        assert_eq!(
            error
                .source()
                .and_then(|source| source.downcast_ref::<io::Error>())
                .map(io::Error::kind),
            Some(io::ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn io_error_returns_none_for_non_io_sources() {
        let error = CodecError::invalid_encoding_for_operation_with_source(
            CodecKind::TextComponent,
            CodecOperation::Read,
            0,
            InvalidEncodingReason::InvalidNbt,
            NonIoSource,
        );
        assert!(error.io_error().is_none());
        assert!(error.source().is_some());
    }

    #[derive(Debug)]
    struct NonIoSource;

    impl fmt::Display for NonIoSource {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("non-io source")
        }
    }

    impl Error for NonIoSource {}
}
