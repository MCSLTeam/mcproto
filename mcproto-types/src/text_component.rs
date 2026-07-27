use std::{
    collections::HashMap,
    error::Error,
    io::{self, Read, Write},
};

use fastnbt::{
    DeOpts, SerOpts,
    stream::{Parser, Value as StreamValue},
};
use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};

use crate::TypeCodec;

/// The contents of a `TAG_Compound` text component.
pub type NbtCompound = HashMap<String, NbtValue>;

/// An owned NBT value provided by `fastnbt`.
pub use fastnbt::Value as NbtValue;

/// A text component in its two valid network NBT root forms.
///
/// Plain text without styling, events, or siblings may use `TAG_String`.
/// Every other component uses `TAG_Compound`.
#[derive(Debug, Clone, PartialEq)]
pub enum TextComponent {
    String(String),
    Compound(NbtCompound),
}

impl TextComponent {
    pub fn text(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn compound(value: NbtCompound) -> Self {
        Self::Compound(value)
    }
}

impl Default for TextComponent {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<NbtCompound> for TextComponent {
    fn from(value: NbtCompound) -> Self {
        Self::Compound(value)
    }
}

impl TypeCodec for TextComponent {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        match self {
            Self::String(value) => encode_string(value, writer),
            Self::Compound(value) => encode_compound(value, writer),
        }
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut reader = CountedReader::new(reader);
        let mut root_tag = [0_u8; 1];
        reader.read_exact(&mut root_tag).map_err(|error| {
            CodecError::from_read_error(CodecKind::TextComponent, reader.processed, error)
        })?;

        match root_tag[0] {
            8 => decode_string(&mut reader).map(Self::String),
            10 => decode_compound(&mut reader).map(Self::Compound),
            tag => Err(CodecError::invalid_encoding(
                CodecKind::TextComponent,
                reader.processed,
                InvalidEncodingReason::InvalidTextComponentRootTag { tag },
            )),
        }
    }
}

fn encode_string(value: &str, writer: &mut impl Write) -> Result<(), CodecError> {
    validate_nbt_string(value, CodecOperation::Write, 0)?;

    // fastnbt only accepts compounds at the Serde root. Wrapping the value in
    // a one-entry compound lets fastnbt perform the NBT string encoding; only
    // the compound framing is removed below.
    let wrapper = HashMap::from([("", value)]);
    let encoded = fastnbt::to_bytes_with_opts(&wrapper, SerOpts::network_nbt())
        .map_err(|error| invalid_nbt(CodecOperation::Write, 0, error))?;

    if encoded.len() < 7
        || encoded[0] != 10
        || encoded[1] != 8
        || encoded[2..4] != [0, 0]
        || encoded[encoded.len() - 1] != 0
    {
        return Err(CodecError::invalid_encoding_for_operation(
            CodecKind::TextComponent,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::InvalidNbt,
        ));
    }

    let payload_len = u16::from_be_bytes([encoded[4], encoded[5]]) as usize;
    if encoded.len() != payload_len + 7 {
        return Err(CodecError::invalid_encoding_for_operation(
            CodecKind::TextComponent,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::StringTooLong {
                max_bytes: u16::MAX as usize,
            },
        ));
    }

    let mut writer = CountedWriter::new(writer);
    writer.write_all(&encoded[1..2]).map_err(|error| {
        CodecError::from_write_error(CodecKind::TextComponent, writer.processed, error)
    })?;
    writer
        .write_all(&encoded[4..encoded.len() - 1])
        .map_err(|error| {
            CodecError::from_write_error(CodecKind::TextComponent, writer.processed, error)
        })
}

fn encode_compound(value: &NbtCompound, writer: &mut impl Write) -> Result<(), CodecError> {
    validate_compound_strings(value)?;

    let mut writer = CountedWriter::new(writer);
    let result = fastnbt::to_writer_with_opts(&mut writer, value, SerOpts::network_nbt());
    match result {
        Ok(()) => Ok(()),
        Err(error) => {
            let processed = writer.processed;
            match writer.failure.take() {
                Some(source) => Err(CodecError::from_write_error(
                    CodecKind::TextComponent,
                    processed,
                    source,
                )),
                None => Err(invalid_nbt(CodecOperation::Write, processed, error)),
            }
        }
    }
}

fn decode_string(reader: &mut CountedReader<'_, impl Read>) -> Result<String, CodecError> {
    // The stream parser expects every root tag to have a name. Network text
    // components omit it, so inject an empty name without consuming input.
    let mut prefixed = PrefixReader {
        prefix: &[8, 0, 0],
        inner: reader,
    };
    let result = Parser::new(&mut prefixed).next();
    match result {
        Ok(StreamValue::String(Some(name), value)) if name.is_empty() => Ok(value),
        Ok(_) => Err(CodecError::invalid_encoding(
            CodecKind::TextComponent,
            prefixed.inner.processed,
            InvalidEncodingReason::InvalidNbt,
        )),
        Err(error) => {
            let processed = prefixed.inner.processed;
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

fn decode_compound(reader: &mut CountedReader<'_, impl Read>) -> Result<NbtCompound, CodecError> {
    let mut prefixed = PrefixReader {
        prefix: &[10],
        inner: reader,
    };
    let result = fastnbt::from_reader_with_opts(&mut prefixed, DeOpts::network_nbt());
    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let processed = prefixed.inner.processed;
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

fn validate_compound_strings(compound: &NbtCompound) -> Result<(), CodecError> {
    let mut pending = Vec::with_capacity(compound.len());
    for (name, value) in compound {
        validate_nbt_string(name, CodecOperation::Write, 0)?;
        pending.push(value);
    }

    while let Some(value) = pending.pop() {
        match value {
            NbtValue::String(value) => {
                validate_nbt_string(value, CodecOperation::Write, 0)?;
            }
            NbtValue::List(values) => pending.extend(values),
            NbtValue::Compound(values) => {
                for (name, value) in values {
                    validate_nbt_string(name, CodecOperation::Write, 0)?;
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
    failure: Option<io::Error>,
}

impl<'a, R: ?Sized> CountedReader<'a, R> {
    fn new(inner: &'a mut R) -> Self {
        Self {
            inner,
            processed: 0,
            failure: None,
        }
    }
}

impl<R: Read + ?Sized> Read for CountedReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        match self.inner.read(buffer) {
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
            Err(error) => {
                self.failure = Some(error);
                let stored = self.failure.as_ref().expect("error was just stored");
                Err(clone_io_error(stored))
            }
        }
    }
}

struct CountedWriter<'a, W: ?Sized> {
    inner: &'a mut W,
    processed: usize,
    failure: Option<io::Error>,
}

impl<'a, W: ?Sized> CountedWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self {
            inner,
            processed: 0,
            failure: None,
        }
    }
}

impl<W: Write + ?Sized> Write for CountedWriter<'_, W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        match self.inner.write(buffer) {
            Ok(0) => {
                let error = io::Error::new(io::ErrorKind::WriteZero, "failed to write NBT");
                self.failure = Some(error);
                Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write NBT",
                ))
            }
            Ok(written) => {
                self.processed += written;
                Ok(written)
            }
            Err(error) => {
                self.failure = Some(error);
                let stored = self.failure.as_ref().expect("error was just stored");
                Err(clone_io_error(stored))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct PrefixReader<'a, 'b, R: ?Sized> {
    prefix: &'static [u8],
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
