//! Packet IDs, frame limits, and the outbound encoding pipeline.

pub mod configuration;
pub mod handshaking;
pub mod login;
pub mod play;
pub mod status;

use std::{
    fmt,
    io::{BufReader, Read},
};

use crate::error::{CompressionError, FrameError, IoOperation, NetworkError, PacketIdError};
use crate::{CompressionMode, DecryptionMode, Direction, EncryptionMode, ProtocolState};
use mcproto_codec::varint::{VarIntRead, VarIntWrite};

/// The maximum packet-frame payload accepted by the default limits.
pub const DEFAULT_MAX_FRAME_LENGTH: usize = (1 << 21) - 1;
/// The maximum uncompressed packet payload accepted by the reader defaults.
///
/// This follows the protocol's serverbound compressed packet limit of 8 MiB.
pub const DEFAULT_MAX_UNCOMPRESSED_LENGTH: usize = 1 << 23;

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

/// Resource limits applied while reading and decompressing packet frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketReadLimits {
    /// Maximum bytes after the outer Packet Length prefix.
    pub max_frame_length: usize,
    /// Maximum bytes after decompression.
    pub max_uncompressed_length: usize,
}

impl Default for PacketReadLimits {
    fn default() -> Self {
        Self {
            max_frame_length: DEFAULT_MAX_FRAME_LENGTH,
            max_uncompressed_length: DEFAULT_MAX_UNCOMPRESSED_LENGTH,
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

/// One decoded packet ID and its body bytes.
///
/// The packet ID prefix remains inside the owned allocation so reading a
/// payload does not require copying the body. Use [`body`](Self::body) for a
/// borrowed body slice or [`into_body`](Self::into_body) to compact the buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketPayload {
    /// Numeric packet ID read from the payload.
    pub packet_id: PacketId,
    data: Vec<u8>,
    body_offset: usize,
}

impl PacketPayload {
    /// Returns the packet body without its ID prefix.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.data[self.body_offset..]
    }

    /// Extracts the packet body without its ID prefix.
    #[must_use]
    pub fn into_body(mut self) -> Vec<u8> {
        self.data.copy_within(self.body_offset.., 0);
        self.data.truncate(self.data.len() - self.body_offset);
        self.data
    }
}

/// Reads, decrypts, decompresses, and parses inbound packet frames.
///
/// The reader owns the inbound cipher state because Minecraft's stream cipher
/// spans packet boundaries. Compression processing is packet-local.
pub struct PacketReader<R> {
    reader: BufReader<R>,
    compression: CompressionMode,
    decryption: DecryptionMode,
    limits: PacketReadLimits,
}

impl<R: Read> PacketReader<R> {
    /// Creates a reader with the negotiated compression, decryption, and limits.
    #[must_use]
    pub fn new(
        reader: R,
        compression: CompressionMode,
        decryption: DecryptionMode,
        limits: PacketReadLimits,
    ) -> Self {
        Self {
            reader: BufReader::new(reader),
            compression,
            decryption,
            limits,
        }
    }

    /// Returns the active compression mode.
    #[must_use]
    pub const fn compression(&self) -> &CompressionMode {
        &self.compression
    }

    /// Replaces compression mode at an explicit protocol boundary.
    pub fn set_compression(&mut self, compression: CompressionMode) {
        self.compression = compression;
    }

    /// Returns the active decryption mode.
    #[must_use]
    pub const fn decryption(&self) -> &DecryptionMode {
        &self.decryption
    }

    /// Replaces decryption mode at an explicit protocol boundary.
    pub fn set_decryption(&mut self, decryption: DecryptionMode) {
        self.decryption = decryption;
    }

    /// Returns the active read limits.
    #[must_use]
    pub const fn limits(&self) -> PacketReadLimits {
        self.limits
    }

    /// Replaces read limits.
    pub fn set_limits(&mut self, limits: PacketReadLimits) {
        self.limits = limits;
    }

    /// Reads one complete packet frame and returns its ID and body.
    ///
    /// A clean end of input before a frame starts is reported as
    /// [`NetworkError::Closed`].
    pub fn read_packet(&mut self) -> Result<PacketPayload, NetworkError> {
        let frame_length = self.read_frame_length()?;
        if frame_length < 0 {
            return Err(FrameError::NegativeLength {
                value: frame_length,
            }
            .into());
        }
        let frame_length = frame_length as usize;
        if frame_length > self.limits.max_frame_length {
            return Err(FrameError::LengthTooLarge {
                actual: frame_length,
                maximum: self.limits.max_frame_length,
            }
            .into());
        }

        let mut frame = vec![0; frame_length];
        let received = self.read_decrypted_exact(&mut frame)?;
        if received != frame_length {
            return Err(NetworkError::TruncatedFrame {
                expected: frame_length,
                received,
            });
        }

        let payload = self.decode_frame(frame)?;
        self.parse_packet(payload)
    }

    fn read_frame_length(&mut self) -> Result<i32, NetworkError> {
        let mut encoded = [0; 5];
        let mut length = 0;

        while length < encoded.len() {
            let read = self.read_decrypted_exact(&mut encoded[length..length + 1])?;
            if read == 0 {
                return if length == 0 {
                    Err(NetworkError::Closed)
                } else {
                    Err(NetworkError::TruncatedFrame {
                        expected: length + 1,
                        received: length,
                    })
                };
            }

            length += 1;
            if encoded[length - 1] & 0x80 == 0 {
                break;
            }
        }

        let mut input = &encoded[..length];
        let (value, _) =
            input
                .read_varint_with_size()
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: None,
                    source,
                })?;
        Ok(value)
    }

    fn read_decrypted_exact(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        let mut read = 0;
        while read < buffer.len() {
            match self.reader.read(&mut buffer[read..]) {
                Ok(0) => break,
                Ok(size) => {
                    self.decryption.decrypt(&mut buffer[read..read + size])?;
                    read += size;
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(source) => {
                    return Err(NetworkError::transport(IoOperation::Read, source));
                }
            }
        }
        Ok(read)
    }

    fn decode_frame(&mut self, frame: Vec<u8>) -> Result<Vec<u8>, NetworkError> {
        let max_uncompressed_length = self.limits.max_uncompressed_length;
        let CompressionMode::Enabled { codec, .. } = &mut self.compression else {
            return Ok(frame);
        };

        if frame.is_empty() {
            return Err(FrameError::MissingDataLength.into());
        }

        let mut input = frame.as_slice();
        let (data_length, prefix_size) =
            input
                .read_varint_with_size()
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: None,
                    source,
                })?;
        if data_length < 0 {
            return Err(FrameError::NegativeLength { value: data_length }.into());
        }
        let data_length = data_length as usize;

        if data_length == 0 {
            let payload_length = frame.len() - prefix_size;
            if payload_length > max_uncompressed_length {
                return Err(CompressionError::OutputTooLarge {
                    actual: payload_length,
                    maximum: max_uncompressed_length,
                }
                .into());
            }
            let mut frame = frame;
            frame.drain(..prefix_size);
            return Ok(frame);
        }

        if data_length > max_uncompressed_length {
            return Err(CompressionError::OutputTooLarge {
                actual: data_length,
                maximum: max_uncompressed_length,
            }
            .into());
        }

        let decompressed = codec.decompress(&frame[prefix_size..], data_length)?;
        if decompressed.len() > max_uncompressed_length {
            return Err(CompressionError::OutputTooLarge {
                actual: decompressed.len(),
                maximum: max_uncompressed_length,
            }
            .into());
        }
        if decompressed.len() != data_length {
            return Err(CompressionError::LengthMismatch {
                declared: data_length,
                actual: decompressed.len(),
            }
            .into());
        }
        Ok(decompressed)
    }

    fn parse_packet(&self, payload: Vec<u8>) -> Result<PacketPayload, NetworkError> {
        if payload.is_empty() {
            return Err(FrameError::MissingPacketId.into());
        }

        let mut input = payload.as_slice();
        let (packet_id, body_offset) =
            input
                .read_varint_with_size()
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: None,
                    source,
                })?;
        let packet_id =
            PacketId::new(packet_id).ok_or(FrameError::InvalidPacketId { value: packet_id })?;

        Ok(PacketPayload {
            packet_id,
            data: payload,
            body_offset,
        })
    }
}
