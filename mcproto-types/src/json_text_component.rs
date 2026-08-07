//! JSON-backed Minecraft text components encoded as protocol strings.
//!
//! This module wraps the shared component model for the JSON representation
//! used by the Minecraft protocol.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};
use serde::{Deserialize, Serialize};

use crate::{
    TypeCodec,
    basic::{decode_prefixed_string, encode_prefixed_string},
    component::JsonComponent,
};

/// The dynamically typed JSON value used by [`JsonComponent`].
pub use serde_json::Value as JsonValue;

/// A text component encoded as JSON in a protocol string.
///
/// Since Java Edition 1.20.3, the vanilla implementation permits up to
/// 262,144 UTF-16 code units when decoding but refuses to encode more than
/// 32,767. See the [text component format].
///
/// [text component format]: https://minecraft.wiki/w/Text_component_format
#[derive(Debug, Clone, PartialEq)]
pub struct JsonTextComponent(
    /// The structured text component value.
    pub JsonComponent,
);

impl JsonTextComponent {
    /// The maximum number of UTF-16 code units accepted while decoding.
    pub const MAX_DECODE_UTF16_CODE_UNITS: usize = 262_144;
    /// The maximum decoded UTF-8 payload size, excluding its VarInt length prefix.
    pub const MAX_DECODE_BYTES: usize = Self::MAX_DECODE_UTF16_CODE_UNITS * 3;
    /// The maximum decoded size, including the VarInt length prefix.
    pub const MAX_DECODE_ENCODED_BYTES: usize = Self::MAX_DECODE_BYTES + 3;

    /// The maximum number of UTF-16 code units permitted while encoding.
    ///
    /// Vanilla Java Edition 1.20.3 and later still refuses to encode larger
    /// JSON components despite accepting them while decoding.
    pub const MAX_ENCODE_UTF16_CODE_UNITS: usize = 32_767;
    /// The maximum encoded UTF-8 payload size, excluding its VarInt length prefix.
    pub const MAX_ENCODE_BYTES: usize = Self::MAX_ENCODE_UTF16_CODE_UNITS * 3;
    /// The maximum encoded size, including the VarInt length prefix.
    pub const MAX_ENCODE_ENCODED_BYTES: usize = Self::MAX_ENCODE_BYTES + 3;

    const MAX_COMPONENT_DEPTH: usize = 512;

    /// Creates a plain-text component.
    pub fn text(value: impl Into<String>) -> Self {
        Self(JsonComponent::text(value))
    }

    /// Parses and validates a text component from JSON without a length prefix.
    ///
    /// On failure, the number of bytes of `value` is reported as
    /// [`CodecError::bytes_processed`].
    ///
    /// # Errors
    ///
    /// Returns a [`CodecError`] if `value` is invalid JSON, does not represent
    /// a text component, or exceeds the supported nesting depth.
    pub fn from_json_str(value: &str) -> Result<Self, CodecError> {
        let component = deserialize_json(value)
            .map_err(|source| Self::invalid_json(CodecOperation::Read, value.len(), source))?;
        validate_component(&component)
            .map_err(|source| Self::invalid_json(CodecOperation::Read, value.len(), source))?;
        Ok(Self(component))
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
        encode_prefixed_string(
            &json,
            writer,
            CodecKind::JsonTextComponent,
            Self::MAX_ENCODE_BYTES,
            Self::MAX_ENCODE_UTF16_CODE_UNITS,
        )
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (json, bytes_processed) = decode_prefixed_string(
            reader,
            CodecKind::JsonTextComponent,
            Self::MAX_DECODE_BYTES,
            Self::MAX_DECODE_UTF16_CODE_UNITS,
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
