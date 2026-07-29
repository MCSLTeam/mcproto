//! NBT-backed Minecraft text components.
//!
//! This module wraps the shared component model for the NBT representation
//! used by the Minecraft protocol.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    io::{self, Read, Write},
};

use fastnbt::{
    SerOpts, Tag,
    stream::{Parser, Value as StreamValue},
};
use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::write_all_counted,
};

use crate::{TypeCodec, component::NbtComponent};

/// The dynamically typed NBT value used by [`NbtComponent`].
pub use fastnbt::Value as NbtValue;

/// A text component encoded as an NBT tag.
///
/// Plain text-only components may use an NBT string tag. Components with
/// styling, events, or other data use an NBT compound tag. See the
/// [text component format] and [NBT specification].
///
/// [text component format]: https://minecraft.wiki/w/Text_component_format
/// [NBT specification]: https://minecraft.wiki/w/NBT_format
#[derive(Debug, Clone, PartialEq)]
pub struct TextComponent(
    /// The structured text component value.
    pub NbtComponent,
);

impl TextComponent {
    /// The maximum number of NBT bytes accepted while decoding a component.
    pub const MAX_DECODE_BYTES: usize = 2 * 1024 * 1024;
    /// The maximum number of NBT values accepted while decoding a component.
    pub const MAX_DECODE_NODES: usize = 262_144;
    const MAX_COMPONENT_DEPTH: usize = 512;

    /// Creates a plain-text component.
    pub fn text(value: impl Into<String>) -> Self {
        Self(NbtComponent::text(value))
    }
}

impl Default for TextComponent {
    fn default() -> Self {
        Self::text("")
    }
}

impl From<NbtComponent> for TextComponent {
    fn from(value: NbtComponent) -> Self {
        Self(value)
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl TypeCodec for TextComponent {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.0
            .validate_depth(Self::MAX_COMPONENT_DEPTH)
            .map_err(|source| invalid_nbt(CodecOperation::Write, 0, source))?;
        self.0
            .validate_dynamic_depth(Self::MAX_COMPONENT_DEPTH)
            .map_err(|source| invalid_nbt(CodecOperation::Write, 0, source))?;
        let normalized = self.0.normalized_root_for_nbt();
        let value = fastnbt::to_value(&normalized)
            .map_err(|source| invalid_nbt(CodecOperation::Write, 0, source))?;
        validate_nbt_strings(&value, CodecOperation::Write, 0)?;
        encode_root(&value, writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (value, bytes_processed) = decode_root(reader)?;
        let component: NbtComponent = fastnbt::from_value(&value)
            .map_err(|source| invalid_nbt(CodecOperation::Read, bytes_processed, source))?;
        component
            .validate_depth(Self::MAX_COMPONENT_DEPTH)
            .map_err(|source| invalid_nbt(CodecOperation::Read, bytes_processed, source))?;
        component
            .validate_dynamic_depth(Self::MAX_COMPONENT_DEPTH)
            .map_err(|source| invalid_nbt(CodecOperation::Read, bytes_processed, source))?;
        Ok(Self(component.normalized_root_for_nbt()))
    }
}

fn encode_root(value: &NbtValue, writer: &mut impl Write) -> Result<(), CodecError> {
    let wrapper = HashMap::from([("", value)]);
    let encoded = fastnbt::to_bytes_with_opts(&wrapper, SerOpts::network_nbt())
        .map_err(|source| invalid_nbt(CodecOperation::Write, 0, source))?;

    if encoded.len() < 5
        || encoded[0] != Tag::Compound as u8
        || encoded[2..4] != [0, 0]
        || encoded.last() != Some(&(Tag::End as u8))
        || !matches!(encoded[1], 8 | 10)
    {
        return Err(CodecError::invalid_encoding_for_operation(
            CodecKind::TextComponent,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::InvalidNbt,
        ));
    }

    write_all_counted(writer, &encoded[1..2], CodecKind::TextComponent, 0)?;
    write_all_counted(
        writer,
        &encoded[4..encoded.len() - 1],
        CodecKind::TextComponent,
        1,
    )
}

fn decode_root(reader: &mut impl Read) -> Result<(NbtValue, usize), CodecError> {
    let mut reader = CountedReader::new(reader, TextComponent::MAX_DECODE_BYTES);
    let mut root_tag = [0_u8; 1];
    reader.read_exact(&mut root_tag).map_err(|source| {
        CodecError::from_read_error(CodecKind::TextComponent, reader.processed, source)
    })?;
    if !matches!(root_tag[0], 8 | 10) {
        return Err(CodecError::invalid_encoding(
            CodecKind::TextComponent,
            reader.processed,
            InvalidEncodingReason::InvalidTextComponentRootTag { tag: root_tag[0] },
        ));
    }

    // The stream parser requires a root name; network text components omit it.
    let mut prefixed = PrefixReader {
        prefix: &[root_tag[0], 0, 0],
        inner: &mut reader,
    };
    let result = (|| {
        let mut parser = Parser::new(&mut prefixed);
        let first = parser.next().map_err(NbtTreeError::Parser)?;
        let mut remaining_nodes = TextComponent::MAX_DECODE_NODES;
        parse_stream_value(&mut parser, first, 1, &mut remaining_nodes)
    })();

    match result {
        Ok(value) => Ok((value, prefixed.inner.processed)),
        Err(error) => {
            let processed = prefixed.inner.processed;
            if prefixed.inner.limit_exceeded {
                return Err(CodecError::invalid_encoding(
                    CodecKind::TextComponent,
                    processed,
                    InvalidEncodingReason::TooLong {
                        max_bytes: TextComponent::MAX_DECODE_BYTES,
                    },
                ));
            }
            match prefixed.inner.failure.take() {
                Some(source) => Err(CodecError::from_read_error(
                    CodecKind::TextComponent,
                    processed,
                    source,
                )),
                None => Err(invalid_nbt(CodecOperation::Read, processed, error)),
            }
        }
    }
}

fn parse_stream_value<R: Read>(
    parser: &mut Parser<R>,
    value: StreamValue,
    depth: usize,
    remaining_nodes: &mut usize,
) -> Result<NbtValue, NbtTreeError> {
    *remaining_nodes = remaining_nodes
        .checked_sub(1)
        .ok_or(NbtTreeError::TooManyNodes)?;
    if depth > TextComponent::MAX_COMPONENT_DEPTH {
        return Err(NbtTreeError::TooDeep);
    }
    match value {
        StreamValue::Byte(_, value) => Ok(NbtValue::Byte(value)),
        StreamValue::Short(_, value) => Ok(NbtValue::Short(value)),
        StreamValue::Int(_, value) => Ok(NbtValue::Int(value)),
        StreamValue::Long(_, value) => Ok(NbtValue::Long(value)),
        StreamValue::Float(_, value) => Ok(NbtValue::Float(value)),
        StreamValue::Double(_, value) => Ok(NbtValue::Double(value)),
        StreamValue::ByteArray(_, value) => Ok(NbtValue::ByteArray(fastnbt::ByteArray::new(value))),
        StreamValue::String(_, value) => Ok(NbtValue::String(value)),
        StreamValue::IntArray(_, value) => Ok(NbtValue::IntArray(fastnbt::IntArray::new(value))),
        StreamValue::LongArray(_, value) => Ok(NbtValue::LongArray(fastnbt::LongArray::new(value))),
        StreamValue::List(_, _, length) => {
            let length = usize::try_from(length).map_err(|_| NbtTreeError::InvalidStructure)?;
            // A declared length is untrusted and can exceed the remaining packet.
            let mut values = Vec::with_capacity(length.min(1024));
            for _ in 0..length {
                let value = parser.next().map_err(NbtTreeError::Parser)?;
                values.push(parse_stream_value(
                    parser,
                    value,
                    depth + 1,
                    remaining_nodes,
                )?);
            }
            if !matches!(parser.next(), Ok(StreamValue::ListEnd)) {
                return Err(NbtTreeError::InvalidStructure);
            }
            Ok(NbtValue::List(values))
        }
        StreamValue::Compound(_) => {
            let mut values = HashMap::new();
            loop {
                let value = parser.next().map_err(NbtTreeError::Parser)?;
                if matches!(value, StreamValue::CompoundEnd) {
                    break;
                }
                let name = stream_name(&value).ok_or(NbtTreeError::InvalidStructure)?;
                values.insert(
                    name.to_owned(),
                    parse_stream_value(parser, value, depth + 1, remaining_nodes)?,
                );
            }
            Ok(NbtValue::Compound(values))
        }
        StreamValue::ListEnd | StreamValue::CompoundEnd => Err(NbtTreeError::InvalidStructure),
    }
}

fn stream_name(value: &StreamValue) -> Option<&str> {
    match value {
        StreamValue::Byte(name, _)
        | StreamValue::Short(name, _)
        | StreamValue::Int(name, _)
        | StreamValue::Long(name, _)
        | StreamValue::Float(name, _)
        | StreamValue::Double(name, _)
        | StreamValue::ByteArray(name, _)
        | StreamValue::String(name, _)
        | StreamValue::List(name, _, _)
        | StreamValue::Compound(name)
        | StreamValue::IntArray(name, _)
        | StreamValue::LongArray(name, _) => name.as_deref(),
        StreamValue::ListEnd | StreamValue::CompoundEnd => None,
    }
}

#[derive(Debug)]
enum NbtTreeError {
    Parser(fastnbt::stream::Error),
    InvalidStructure,
    TooDeep,
    TooManyNodes,
}

impl fmt::Display for NbtTreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parser(source) => source.fmt(formatter),
            Self::InvalidStructure => formatter.write_str("invalid NBT tree structure"),
            Self::TooDeep => formatter.write_str("NBT tree is nested too deeply"),
            Self::TooManyNodes => formatter.write_str("NBT tree contains too many nodes"),
        }
    }
}

impl Error for NbtTreeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parser(source) => Some(source),
            _ => None,
        }
    }
}

fn invalid_nbt(
    operation: CodecOperation,
    bytes_processed: usize,
    source: impl Error + Send + Sync + 'static,
) -> CodecError {
    CodecError::invalid_encoding_for_operation_with_source(
        CodecKind::TextComponent,
        operation,
        bytes_processed,
        InvalidEncodingReason::InvalidNbt,
        source,
    )
}

fn validate_nbt_strings(
    root: &NbtValue,
    operation: CodecOperation,
    bytes_processed: usize,
) -> Result<(), CodecError> {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            NbtValue::String(value) => validate_nbt_string(value, operation, bytes_processed)?,
            NbtValue::List(values) => pending.extend(values),
            NbtValue::Compound(values) => {
                for (name, value) in values {
                    validate_nbt_string(name, operation, bytes_processed)?;
                    pending.push(value);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_nbt_string(
    value: &str,
    operation: CodecOperation,
    bytes_processed: usize,
) -> Result<(), CodecError> {
    let encoded_len = value.chars().try_fold(0_usize, |length, character| {
        let width = match character as u32 {
            0 => 2,
            1..=0x7f => 1,
            0x80..=0x7ff => 2,
            0x800..=0xffff => 3,
            _ => 6,
        };
        length.checked_add(width)
    });
    if !matches!(encoded_len, Some(length) if length <= u16::MAX as usize) {
        return Err(CodecError::invalid_encoding_for_operation(
            CodecKind::TextComponent,
            operation,
            bytes_processed,
            InvalidEncodingReason::StringTooLong {
                max_bytes: u16::MAX as usize,
            },
        ));
    }
    Ok(())
}

struct CountedReader<'a, R: ?Sized> {
    inner: &'a mut R,
    processed: usize,
    byte_limit: usize,
    limit_exceeded: bool,
    failure: Option<io::Error>,
}

impl<'a, R: ?Sized> CountedReader<'a, R> {
    fn new(inner: &'a mut R, byte_limit: usize) -> Self {
        Self {
            inner,
            processed: 0,
            byte_limit,
            limit_exceeded: false,
            failure: None,
        }
    }
}

impl<R: Read + ?Sized> Read for CountedReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let remaining = self.byte_limit.saturating_sub(self.processed);
        if remaining == 0 {
            self.limit_exceeded = true;
            return Err(io::Error::other("NBT byte limit exceeded"));
        }
        let buffer_len = buffer.len().min(remaining);
        match self.inner.read(&mut buffer[..buffer_len]) {
            Ok(0) => {
                let error = io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected end of NBT");
                self.failure = Some(error);
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected end of NBT",
                ))
            }
            Ok(read) => {
                self.processed += read;
                Ok(read)
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Err(error),
            Err(error) => {
                let returned = clone_io_error(&error);
                self.failure = Some(error);
                Err(returned)
            }
        }
    }
}

struct PrefixReader<'a, 'b, R: ?Sized> {
    prefix: &'a [u8],
    inner: &'a mut CountedReader<'b, R>,
}

impl<R: Read + ?Sized> Read for PrefixReader<'_, '_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if !self.prefix.is_empty() {
            let read = buffer.len().min(self.prefix.len());
            buffer[..read].copy_from_slice(&self.prefix[..read]);
            self.prefix = &self.prefix[read..];
            return Ok(read);
        }
        self.inner.read(buffer)
    }
}

fn clone_io_error(error: &io::Error) -> io::Error {
    match error.raw_os_error() {
        Some(code) => io::Error::from_raw_os_error(code),
        None => io::Error::new(error.kind(), error.to_string()),
    }
}
