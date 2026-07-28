use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
};
use serde::{Deserialize, Serialize};

use crate::{TypeCodec, component::JsonComponent};

pub use serde_json::Value as JsonValue;

/// A VarInt-prefixed JSON representation of the current Java text component schema.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonTextComponent(pub JsonComponent);

impl JsonTextComponent {
    pub const MAX_DECODE_UTF16_CODE_UNITS: usize = 262_144;
    pub const MAX_DECODE_BYTES: usize = Self::MAX_DECODE_UTF16_CODE_UNITS * 3;
    pub const MAX_DECODE_ENCODED_BYTES: usize = Self::MAX_DECODE_BYTES + 3;

    // Vanilla 1.20.3+ still refuses to encode larger JSON components.
    pub const MAX_ENCODE_UTF16_CODE_UNITS: usize = 32_767;
    pub const MAX_ENCODE_BYTES: usize = Self::MAX_ENCODE_UTF16_CODE_UNITS * 3;
    pub const MAX_ENCODE_ENCODED_BYTES: usize = Self::MAX_ENCODE_BYTES + 3;

    const MAX_COMPONENT_DEPTH: usize = 512;

    pub fn text(value: impl Into<String>) -> Self {
        Self(JsonComponent::text(value))
    }

    pub fn from_json_str(value: &str) -> Result<Self, serde_json::Error> {
        let component = deserialize_json(value)?;
        validate_component(&component).map_err(json_validation_error)?;
        Ok(Self(component))
    }

    fn validate_length(
        value: &str,
        max_code_units: usize,
        max_bytes: usize,
        operation: CodecOperation,
        bytes_processed: usize,
    ) -> Result<(), CodecError> {
        if value.len() > max_bytes {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::JsonTextComponent,
                operation,
                bytes_processed,
                InvalidEncodingReason::StringTooLong { max_bytes },
            ));
        }
        if value.encode_utf16().count() > max_code_units {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::JsonTextComponent,
                operation,
                bytes_processed,
                InvalidEncodingReason::TooManyUtf16CodeUnits { max_code_units },
            ));
        }
        Ok(())
    }

    fn invalid_json(
        operation: CodecOperation,
        bytes_processed: usize,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> CodecError {
        CodecError::invalid_encoding_for_operation_with_source(
            CodecKind::JsonTextComponent,
            operation,
            bytes_processed,
            InvalidEncodingReason::InvalidJson,
            source,
        )
    }
}

impl Default for JsonTextComponent {
    fn default() -> Self {
        Self::text("")
    }
}

impl From<JsonComponent> for JsonTextComponent {
    fn from(value: JsonComponent) -> Self {
        Self(value)
    }
}

impl From<String> for JsonTextComponent {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<&str> for JsonTextComponent {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl TypeCodec for JsonTextComponent {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        validate_component(&self.0)
            .map_err(|source| Self::invalid_json(CodecOperation::Write, 0, source))?;
        let mut bytes = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut bytes);
        self.0
            .serialize(serde_stacker::Serializer::new(&mut serializer))
            .map_err(|source| Self::invalid_json(CodecOperation::Write, 0, source))?;
        let json = String::from_utf8(bytes)
            .map_err(|source| Self::invalid_json(CodecOperation::Write, 0, source))?;
        Self::validate_length(
            &json,
            Self::MAX_ENCODE_UTF16_CODE_UNITS,
            Self::MAX_ENCODE_BYTES,
            CodecOperation::Write,
            0,
        )?;

        let bytes = json.as_bytes();
        let prefix_size = writer
            .write_varint_with_size(bytes.len() as i32)
            .map_err(|error| error.with_context(CodecKind::JsonTextComponent))?;
        write_all_counted(writer, bytes, CodecKind::JsonTextComponent, prefix_size)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (byte_length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::JsonTextComponent))?;
        let byte_length = usize::try_from(byte_length).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::JsonTextComponent,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: byte_length },
            )
        })?;
        if byte_length > Self::MAX_DECODE_BYTES {
            return Err(CodecError::invalid_encoding(
                CodecKind::JsonTextComponent,
                prefix_size,
                InvalidEncodingReason::StringTooLong {
                    max_bytes: Self::MAX_DECODE_BYTES,
                },
            ));
        }

        let mut bytes = vec![0; byte_length];
        read_exact_counted(
            reader,
            &mut bytes,
            CodecKind::JsonTextComponent,
            prefix_size,
        )?;
        let bytes_processed = prefix_size + byte_length;
        let json = String::from_utf8(bytes).map_err(|error| {
            let utf8_error = error.utf8_error();
            CodecError::invalid_encoding(
                CodecKind::JsonTextComponent,
                bytes_processed,
                InvalidEncodingReason::InvalidUtf8 {
                    valid_up_to: utf8_error.valid_up_to(),
                    error_len: utf8_error.error_len(),
                },
            )
        })?;
        Self::validate_length(
            &json,
            Self::MAX_DECODE_UTF16_CODE_UNITS,
            Self::MAX_DECODE_BYTES,
            CodecOperation::Read,
            bytes_processed,
        )?;

        let component = deserialize_json(&json)
            .map_err(|source| Self::invalid_json(CodecOperation::Read, bytes_processed, source))?;
        validate_component(&component)
            .map_err(|source| Self::invalid_json(CodecOperation::Read, bytes_processed, source))?;
        Ok(Self(component))
    }
}

fn deserialize_json(value: &str) -> Result<JsonComponent, serde_json::Error> {
    validate_json_syntax_depth(value, JsonTextComponent::MAX_COMPONENT_DEPTH * 2 + 8)
        .map_err(json_validation_error)?;
    let mut deserializer = serde_json::Deserializer::from_str(value);
    deserializer.disable_recursion_limit();
    let component =
        JsonComponent::deserialize(serde_stacker::Deserializer::new(&mut deserializer))?;
    deserializer.end()?;
    Ok(component)
}

fn validate_json_syntax_depth(value: &str, max_depth: usize) -> Result<(), JsonSyntaxDepthError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for byte in value.bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > max_depth {
                    return Err(JsonSyntaxDepthError { max_depth });
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

#[derive(Debug)]
struct JsonSyntaxDepthError {
    max_depth: usize,
}

impl std::fmt::Display for JsonSyntaxDepthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "JSON nesting exceeds the {}-container limit",
            self.max_depth
        )
    }
}

impl std::error::Error for JsonSyntaxDepthError {}

fn validate_component(
    component: &JsonComponent,
) -> Result<(), crate::component::ComponentDepthError> {
    component.validate_depth(JsonTextComponent::MAX_COMPONENT_DEPTH)?;
    component.validate_dynamic_depth(JsonTextComponent::MAX_COMPONENT_DEPTH)
}

fn json_validation_error(
    source: impl std::error::Error + Send + Sync + 'static,
) -> serde_json::Error {
    serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, source))
}
