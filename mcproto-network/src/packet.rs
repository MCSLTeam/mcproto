//! Packet IDs, frame limits, and the outbound encoding pipeline.

pub mod handshaking;

use std::fmt;

use crate::error::{FrameError, NetworkError, PacketIdError};
use crate::{CompressionMode, Direction, EncryptionMode, ProtocolState};
use mcproto_codec::varint::VarIntWrite;

/// The maximum packet-frame payload accepted by the default limits.
pub const DEFAULT_MAX_FRAME_LENGTH: usize = (1 << 21) - 1;

/// A validated non-negative Minecraft packet ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PacketId(i32);

impl PacketId {
    /// Creates an ID, rejecting negative VarInt values.
    #[must_use]
    pub const fn new(value: i32) -> Option<Self> {
        if value < 0 { None } else { Some(Self(value)) }
    }

    /// Returns the signed value used by the wire VarInt codec.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for PacketId {
    type Error = PacketIdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(PacketIdError { value })
    }
}

impl fmt::Display for PacketId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A stable packet name matching the official protocol documentation.
///
/// Names are metadata and are not written to the Minecraft wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PacketName(&'static str);

impl PacketName {
    /// Creates a static packet name in the official `lower_case` form.
    ///
    /// # Panics
    ///
    /// Panics when `value` is not formed from lowercase ASCII words separated
    /// by single underscores. When used in an associated constant this is
    /// reported at compile time.
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        assert!(
            is_valid_packet_name(value),
            "packet name must use the official lower_case form"
        );
        Self(value)
    }

    /// Returns the name as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

const fn is_valid_packet_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }

    let mut index = 1;
    let mut previous_was_underscore = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'_' {
            if previous_was_underscore {
                return false;
            }
            previous_was_underscore = true;
        } else if byte.is_ascii_lowercase() {
            previous_was_underscore = false;
        } else {
            return false;
        }
        index += 1;
    }
    !previous_was_underscore
}

impl fmt::Display for PacketName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// Metadata shared by packets in either protocol direction.
///
/// Packet types carry the numeric ID for the protocol version they implement.
pub trait Packet {
    /// Stable name matching the official packet name.
    const NAME: PacketName;
    /// Numeric packet ID written as a VarInt on the wire.
    const ID: PacketId;
    /// Protocol state in which the packet is valid.
    const STATE: ProtocolState;
    /// Direction in which the packet is valid.
    const DIRECTION: Direction;
}

/// A serverbound packet that can be encoded by a client.
///
/// Clientbound derives do not implement this trait and therefore cannot be
/// passed to [`PacketEncoder::encode`]:
///
/// ```compile_fail
/// use mcproto_network::{
///     CompressionMode, EncryptionMode, PacketCodec, PacketEncoder, PacketLimits,
/// };
///
/// #[derive(PacketCodec)]
/// #[packet(name = "example", id = 0x00, state = Play, direction = Clientbound)]
/// struct Example;
///
/// let mut encoder = PacketEncoder::new(
///     CompressionMode::disabled(),
///     EncryptionMode::disabled(),
///     PacketLimits::default(),
/// );
/// encoder.encode(&Example).unwrap();
/// ```
pub trait EncodePacket: Packet {
    /// Encodes only this packet's data fields, without its ID or frame length.
    fn encode_body(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), mcproto_codec::error::CodecError>;

    /// Encodes this packet's complete network frame.
    fn encode_packet(&self, encoder: &mut PacketEncoder) -> Result<Vec<u8>, NetworkError>
    where
        Self: Sized,
    {
        encoder.encode(self)
    }
}

/// A clientbound packet that can be decoded by a client.
///
/// Serverbound derives do not implement this trait:
///
/// ```compile_fail
/// use mcproto_network::{DecodePacket, PacketCodec};
///
/// #[derive(PacketCodec)]
/// #[packet(name = "example", id = 0x00, state = Play, direction = Serverbound)]
/// struct Example;
///
/// let mut input = [].as_slice();
/// let _ = Example::decode_body(&mut input).unwrap();
/// ```
pub trait DecodePacket: Packet + Sized {
    /// Decodes this packet's data fields after its ID has been consumed.
    fn decode_body(
        reader: &mut impl std::io::Read,
    ) -> Result<Self, mcproto_codec::error::CodecError>;
}

/// Resource limits applied to outbound frame construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketLimits {
    /// Maximum bytes after the outer Packet Length prefix.
    pub max_frame_length: usize,
}

impl Default for PacketLimits {
    fn default() -> Self {
        Self {
            max_frame_length: DEFAULT_MAX_FRAME_LENGTH,
        }
    }
}

/// Encodes packet bodies, applies compression framing, then encrypts frames.
pub struct PacketEncoder {
    compression: CompressionMode,
    encryption: EncryptionMode,
    limits: PacketLimits,
}

impl PacketEncoder {
    /// Creates an encoder with the supplied negotiated modes and limits.
    #[must_use]
    pub const fn new(
        compression: CompressionMode,
        encryption: EncryptionMode,
        limits: PacketLimits,
    ) -> Self {
        Self {
            compression,
            encryption,
            limits,
        }
    }

    /// Returns the active compression mode.
    #[must_use]
    pub const fn compression(&self) -> &CompressionMode {
        &self.compression
    }

    /// Returns the active encryption mode.
    #[must_use]
    pub const fn encryption(&self) -> &EncryptionMode {
        &self.encryption
    }

    /// Replaces compression mode at an explicit protocol boundary.
    pub fn set_compression(&mut self, compression: CompressionMode) {
        self.compression = compression;
    }

    /// Replaces encryption mode at an explicit protocol boundary.
    pub fn set_encryption(&mut self, encryption: EncryptionMode) {
        self.encryption = encryption;
    }

    /// Encodes a packet using the ID declared by its type.
    pub fn encode<P: EncodePacket>(&mut self, packet: &P) -> Result<Vec<u8>, NetworkError> {
        self.encode_with_id(P::ID, packet)
    }

    /// Encodes a body with a caller-supplied packet ID.
    pub fn encode_with_id<T: EncodePacket>(
        &mut self,
        packet_id: PacketId,
        packet: &T,
    ) -> Result<Vec<u8>, NetworkError> {
        let mut payload = Vec::new();
        payload
            .write_varint(packet_id.get())
            .map_err(|source| NetworkError::PacketCodec {
                packet_id: Some(packet_id.get()),
                source,
            })?;
        packet
            .encode_body(&mut payload)
            .map_err(|source| NetworkError::PacketCodec {
                packet_id: Some(packet_id.get()),
                source,
            })?;

        if payload.len() > self.limits.max_frame_length {
            return Err(NetworkError::Frame(FrameError::LengthTooLarge {
                actual: payload.len(),
                maximum: self.limits.max_frame_length,
            }));
        }

        let frame_data = self.frame_data(&payload)?;
        if frame_data.len() > self.limits.max_frame_length {
            return Err(NetworkError::Frame(FrameError::LengthTooLarge {
                actual: frame_data.len(),
                maximum: self.limits.max_frame_length,
            }));
        }

        let frame_length = i32::try_from(frame_data.len()).map_err(|_| {
            NetworkError::Frame(FrameError::LengthTooLarge {
                actual: frame_data.len(),
                maximum: i32::MAX as usize,
            })
        })?;

        let mut frame = Vec::new();
        frame
            .write_varint(frame_length)
            .map_err(|source| NetworkError::PacketCodec {
                packet_id: Some(packet_id.get()),
                source,
            })?;
        frame.extend_from_slice(&frame_data);

        // Encryption is deliberately last: Packet Length is encrypted too.
        self.encryption.encrypt(&mut frame)?;
        Ok(frame)
    }

    fn frame_data(&mut self, payload: &[u8]) -> Result<Vec<u8>, NetworkError> {
        match &mut self.compression {
            CompressionMode::Disabled => Ok(payload.to_vec()),
            CompressionMode::Enabled { threshold, codec } => {
                let mut data = Vec::new();
                if payload.len() >= *threshold {
                    let uncompressed_length = i32::try_from(payload.len()).map_err(|_| {
                        NetworkError::Frame(FrameError::LengthTooLarge {
                            actual: payload.len(),
                            maximum: i32::MAX as usize,
                        })
                    })?;
                    data.write_varint(uncompressed_length).map_err(|source| {
                        NetworkError::PacketCodec {
                            packet_id: None,
                            source,
                        }
                    })?;
                    let compressed = codec.compress(payload)?;
                    data.extend_from_slice(&compressed);
                } else {
                    data.write_varint(0)
                        .map_err(|source| NetworkError::PacketCodec {
                            packet_id: None,
                            source,
                        })?;
                    data.extend_from_slice(payload);
                }
                Ok(data)
            }
        }
    }
}
