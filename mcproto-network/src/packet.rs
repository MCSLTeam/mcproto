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
    payload_scratch: Vec<u8>,
    frame_scratch: Vec<u8>,
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
            payload_scratch: Vec::new(),
            frame_scratch: Vec::new(),
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

    /// Encodes a packet into a reusable output buffer.
    ///
    /// The output is cleared before encoding and retains its allocation for
    /// subsequent calls.
    pub fn encode_into<P: EncodePacket>(
        &mut self,
        packet: &P,
        output: &mut Vec<u8>,
    ) -> Result<(), NetworkError> {
        self.encode_with_id_into(P::ID, packet, output)
    }

    /// Encodes a body with a caller-supplied packet ID.
    pub fn encode_with_id<T: EncodePacket>(
        &mut self,
        packet_id: PacketId,
        packet: &T,
    ) -> Result<Vec<u8>, NetworkError> {
        let mut frame = Vec::new();
        self.encode_with_id_into(packet_id, packet, &mut frame)?;
        Ok(frame)
    }

    /// Encodes a packet with a caller-supplied ID into a reusable buffer.
    pub fn encode_with_id_into<T: EncodePacket>(
        &mut self,
        packet_id: PacketId,
        packet: &T,
        output: &mut Vec<u8>,
    ) -> Result<(), NetworkError> {
        output.clear();
        if self.compression.is_enabled() {
            self.encode_compressed_into(packet_id, packet, output)?;
        } else {
            self.encode_uncompressed_into(packet_id, packet, output)?;
        }
        // Encryption is deliberately last: Packet Length is encrypted too.
        self.encryption.encrypt(output)?;
        Ok(())
    }

    fn encode_uncompressed_into<T: EncodePacket>(
        &mut self,
        packet_id: PacketId,
        packet: &T,
        output: &mut Vec<u8>,
    ) -> Result<(), NetworkError> {
        const LENGTH_PREFIX_RESERVE: usize = 5;

        output.resize(LENGTH_PREFIX_RESERVE, 0);
        write_packet_payload(output, packet_id, packet)?;
        self.finish_reserved_frame(output, packet_id)
    }

    fn encode_compressed_into<T: EncodePacket>(
        &mut self,
        packet_id: PacketId,
        packet: &T,
        output: &mut Vec<u8>,
    ) -> Result<(), NetworkError> {
        self.payload_scratch.clear();
        write_packet_payload(&mut self.payload_scratch, packet_id, packet)?;
        if self.payload_scratch.len() > self.limits.max_frame_length {
            return Err(NetworkError::Frame(FrameError::LengthTooLarge {
                actual: self.payload_scratch.len(),
                maximum: self.limits.max_frame_length,
            }));
        }

        self.frame_scratch.clear();
        let Self {
            payload_scratch,
            frame_scratch,
            compression,
            ..
        } = self;
        let CompressionMode::Enabled { threshold, codec } = compression else {
            unreachable!("compressed encoding requires enabled compression");
        };
        if payload_scratch.len() >= *threshold {
            let uncompressed_length = i32::try_from(payload_scratch.len()).map_err(|_| {
                NetworkError::Frame(FrameError::LengthTooLarge {
                    actual: payload_scratch.len(),
                    maximum: i32::MAX as usize,
                })
            })?;
            frame_scratch
                .write_varint(uncompressed_length)
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: None,
                    source,
                })?;
            codec.compress_into(payload_scratch, frame_scratch)?;
        } else {
            frame_scratch
                .write_varint(0)
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: None,
                    source,
                })?;
            frame_scratch.extend_from_slice(payload_scratch);
        }
        if self.frame_scratch.len() > self.limits.max_frame_length {
            return Err(NetworkError::Frame(FrameError::LengthTooLarge {
                actual: self.frame_scratch.len(),
                maximum: self.limits.max_frame_length,
            }));
        }

        std::mem::swap(output, &mut self.frame_scratch);
        let frame_data_length = output.len();
        output.reserve(5);
        output.resize(frame_data_length + 5, 0);
        output.copy_within(0..frame_data_length, 5);
        self.finish_reserved_frame(output, packet_id)
    }

    fn finish_reserved_frame(
        &self,
        frame: &mut Vec<u8>,
        packet_id: PacketId,
    ) -> Result<(), NetworkError> {
        const LENGTH_PREFIX_RESERVE: usize = 5;

        let frame_length = frame.len() - LENGTH_PREFIX_RESERVE;
        if frame_length > self.limits.max_frame_length {
            return Err(NetworkError::Frame(FrameError::LengthTooLarge {
                actual: frame_length,
                maximum: self.limits.max_frame_length,
            }));
        }
        let frame_length = i32::try_from(frame_length).map_err(|_| {
            NetworkError::Frame(FrameError::LengthTooLarge {
                actual: frame_length,
                maximum: i32::MAX as usize,
            })
        })?;

        let mut prefix = [0; LENGTH_PREFIX_RESERVE];
        let mut prefix_writer = prefix.as_mut_slice();
        let prefix_length =
            prefix_writer
                .write_varint_with_size(frame_length)
                .map_err(|source| NetworkError::PacketCodec {
                    packet_id: Some(packet_id.get()),
                    source,
                })?;

        frame.copy_within(LENGTH_PREFIX_RESERVE.., prefix_length);
        frame.truncate(prefix_length + frame_length as usize);
        frame[..prefix_length].copy_from_slice(&prefix[..prefix_length]);
        Ok(())
    }
}

fn write_packet_payload<T: EncodePacket>(
    output: &mut Vec<u8>,
    packet_id: PacketId,
    packet: &T,
) -> Result<(), NetworkError> {
    output
        .write_varint(packet_id.get())
        .map_err(|source| NetworkError::PacketCodec {
            packet_id: Some(packet_id.get()),
            source,
        })?;
    packet
        .encode_body(output)
        .map_err(|source| NetworkError::PacketCodec {
            packet_id: Some(packet_id.get()),
            source,
        })
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
    frame_scratch: Vec<u8>,
    payload_scratch: Vec<u8>,
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
            frame_scratch: Vec::new(),
            payload_scratch: Vec::new(),
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
        let (packet_id, body_offset) = self.read_packet_payload()?;
        Ok(PacketPayload {
            packet_id,
            data: std::mem::take(&mut self.payload_scratch),
            body_offset,
        })
    }

    /// Reads one packet into a reusable body buffer and returns its packet ID.
    ///
    /// The output is replaced with the packet body, excluding the packet ID.
    /// The output and internal frame/decompression buffers retain their
    /// allocations for subsequent calls.
    pub fn read_packet_into(&mut self, output: &mut Vec<u8>) -> Result<PacketId, NetworkError> {
        let (packet_id, body_offset) = self.read_packet_payload()?;
        output.clear();
        output.extend_from_slice(&self.payload_scratch[body_offset..]);
        Ok(packet_id)
    }

    fn read_packet_payload(&mut self) -> Result<(PacketId, usize), NetworkError> {
        self.read_frame_into_scratch()?;
        self.decode_frame()?;
        parse_packet_payload(&self.payload_scratch)
    }

    fn read_frame_into_scratch(&mut self) -> Result<(), NetworkError> {
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

        self.frame_scratch.clear();
        self.frame_scratch.resize(frame_length, 0);
        let received = read_decrypted_exact(
            &mut self.reader,
            &mut self.decryption,
            &mut self.frame_scratch,
        )?;
        if received != frame_length {
            return Err(NetworkError::TruncatedFrame {
                expected: frame_length,
                received,
            });
        }
        Ok(())
    }

    fn read_frame_length(&mut self) -> Result<i32, NetworkError> {
        let mut encoded = [0; 5];
        let mut length = 0;

        while length < encoded.len() {
            let read = read_decrypted_exact(
                &mut self.reader,
                &mut self.decryption,
                &mut encoded[length..length + 1],
            )?;
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

    fn decode_frame(&mut self) -> Result<(), NetworkError> {
        let max_uncompressed_length = self.limits.max_uncompressed_length;
        if !self.compression.is_enabled() {
            std::mem::swap(&mut self.payload_scratch, &mut self.frame_scratch);
            return Ok(());
        }

        if self.frame_scratch.is_empty() {
            return Err(FrameError::MissingDataLength.into());
        }

        let mut input = self.frame_scratch.as_slice();
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
            let payload_length = self.frame_scratch.len() - prefix_size;
            if payload_length > max_uncompressed_length {
                return Err(CompressionError::OutputTooLarge {
                    actual: payload_length,
                    maximum: max_uncompressed_length,
                }
                .into());
            }
            std::mem::swap(&mut self.payload_scratch, &mut self.frame_scratch);
            self.payload_scratch.drain(..prefix_size);
            return Ok(());
        }

        if data_length > max_uncompressed_length {
            return Err(CompressionError::OutputTooLarge {
                actual: data_length,
                maximum: max_uncompressed_length,
            }
            .into());
        }

        self.payload_scratch.clear();
        let Self {
            frame_scratch,
            payload_scratch,
            compression,
            ..
        } = self;
        let CompressionMode::Enabled { codec, .. } = compression else {
            unreachable!("compressed decoding requires enabled compression");
        };
        codec.decompress_into(&frame_scratch[prefix_size..], data_length, payload_scratch)?;
        if payload_scratch.len() > max_uncompressed_length {
            return Err(CompressionError::OutputTooLarge {
                actual: payload_scratch.len(),
                maximum: max_uncompressed_length,
            }
            .into());
        }
        if payload_scratch.len() != data_length {
            return Err(CompressionError::LengthMismatch {
                declared: data_length,
                actual: payload_scratch.len(),
            }
            .into());
        }
        Ok(())
    }
}

fn read_decrypted_exact<R: Read>(
    reader: &mut BufReader<R>,
    decryption: &mut DecryptionMode,
    buffer: &mut [u8],
) -> Result<usize, NetworkError> {
    let mut read = 0;
    while read < buffer.len() {
        match reader.read(&mut buffer[read..]) {
            Ok(0) => break,
            Ok(size) => {
                decryption.decrypt(&mut buffer[read..read + size])?;
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

fn parse_packet_payload(payload: &[u8]) -> Result<(PacketId, usize), NetworkError> {
    if payload.is_empty() {
        return Err(FrameError::MissingPacketId.into());
    }

    let mut input = payload;
    let (packet_id, body_offset) =
        input
            .read_varint_with_size()
            .map_err(|source| NetworkError::PacketCodec {
                packet_id: None,
                source,
            })?;
    let packet_id =
        PacketId::new(packet_id).ok_or(FrameError::InvalidPacketId { value: packet_id })?;
    Ok((packet_id, body_offset))
}
