//! Status state packets.

use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};
use mcproto_types::{JsonComponent, Long, PrefixedString, TypeCodec, Uuid};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::PacketCodec;

#[derive(PacketCodec)]
#[packet(
    name = "status_request",
    id = 0x00,
    state = Status,
    direction = Serverbound,
)]
/// Requests the server-list status immediately after handshaking.
pub struct StatusRequest;

/// Version information advertised by a server-list status response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusVersion {
    /// Human-readable Minecraft version name.
    ///
    /// Modern clients tolerate this field being absent and display `Old`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Numeric protocol version supported by the server.
    pub protocol: i32,
}

/// One player shown in the status player-count tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusPlayerSample {
    /// Player name displayed by the client.
    pub name: String,
    /// Player UUID, if supplied by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
}

/// Player counts and the optional status tooltip sample.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusPlayers {
    /// Maximum number of players accepted by the server.
    pub max: i32,
    /// Number of players currently online.
    pub online: i32,
    /// Players displayed in the hover tooltip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<StatusPlayerSample>>,
}

/// Structured contents of the Status Response JSON string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusResponseData {
    /// Server version information.
    pub version: StatusVersion,
    /// Player counts, omitted when the server hides them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusPlayers>,
    /// Message of the day as a JSON text component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<JsonComponent>,
    /// Optional `data:image/png;base64,...` 64x64 server icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    /// Whether the server enforces secure chat signing.
    #[serde(
        rename = "enforcesSecureChat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enforces_secure_chat: Option<bool>,
}

impl TypeCodec for StatusResponseData {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let json = serde_json::to_string(self).map_err(|source| {
            CodecError::invalid_encoding_for_operation_with_source(
                CodecKind::String,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::InvalidJson,
                source,
            )
        })?;
        PrefixedString(json).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let json = PrefixedString::decode(reader)?.0;
        let bytes_processed = encoded_string_len(json.len());
        serde_json::from_str(&json).map_err(|source| {
            CodecError::invalid_encoding_for_operation_with_source(
                CodecKind::String,
                CodecOperation::Read,
                bytes_processed,
                InvalidEncodingReason::InvalidJson,
                source,
            )
        })
    }
}

fn encoded_string_len(payload_len: usize) -> usize {
    let mut value = payload_len;
    let mut prefix_len = 1;
    while value >= 0x80 {
        value >>= 7;
        prefix_len += 1;
    }
    prefix_len + payload_len
}

#[derive(PacketCodec)]
#[packet(
    name = "status_response",
    id = 0x00,
    state = Status,
    direction = Clientbound,
)]
/// Responds to a status request with information about the server.
pub struct StatusResponse {
    /// Parsed server information from the JSON Response field.
    pub json_response: StatusResponseData,
}

#[derive(PacketCodec)]
#[packet(
    name = "ping_request",
    id = 0x01,
    state = Status,
    direction = Serverbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Request_(status))
pub struct PingRequestStatus {
    /// May be any number, but vanilla clients will always use the timestamp in milliseconds.
    pub timestamp: Long,
}
impl PingRequestStatus {
    /// Creates a new `PingRequestStatus` with the current system time in normal seconds.
    pub fn new() -> Self {
        Self {
            timestamp: Long(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backward")
                    .as_secs() as i64,
            ),
        }
    }
}

#[derive(PacketCodec)]
#[packet(
    name = "pong_response",
    id = 0x01,
    state = Status,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong_Response_(status))
pub struct PongResponseStatus {
    /// Should match the one sent by the client.
    pub timestamp: Long,
}
