//! Framing and transport-state primitives for the Minecraft Java protocol.
//!
//! This crate deliberately keeps compression and encryption as replaceable
//! algorithms. The protocol pipeline is fixed:
//!
//! ```text
//! packet body -> packet framing/compression -> stream encryption -> transport
//! ```

#![warn(missing_docs)]

extern crate self as mcproto_network;

mod error;
pub mod packet;
mod state;

pub use error::{
    CompressionError, EncryptionError, ErrorKind, FrameError, IoOperation, NetworkError,
    PacketIdError, StateError,
};
pub use mcproto_derive::PacketCodec;
pub use packet::{
    DecodePacket, EncodePacket, Packet, PacketEncoder, PacketId, PacketLimits, PacketName,
};
pub use state::{CompressionMode, Direction, EncryptionMode, ProtocolState};

/// Re-exports used by generated `PacketCodec` implementations.
#[doc(hidden)]
pub use mcproto_types as __types;

/// A stateful compressor used by the protocol compression layer.
///
/// Implementations should use the Minecraft wire format: zlib-wrapped DEFLATE
/// for the production protocol. The same object may be used for many packets;
/// unlike encryption, compression itself normally does not carry state between
/// packets.
pub trait CompressionCodec {
    /// Compresses one complete `Packet ID + Packet Data` value.
    fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;

    /// Decompresses one compressed packet payload.
    ///
    /// `expected_len` is the value carried by the protocol's Data Length field.
    fn decompress(
        &mut self,
        input: &[u8],
        expected_len: usize,
    ) -> Result<Vec<u8>, CompressionError>;
}

/// A stateful encryptor for one direction of the connection.
///
/// Minecraft uses AES/CFB8. The implementation must retain its feedback state
/// across calls; it must not reinitialize the cipher for every packet.
pub trait StreamEncryptor {
    /// Encrypts bytes in place.
    fn encrypt(&mut self, data: &mut [u8]) -> Result<(), EncryptionError>;
}

/// A stateful decryptor for one direction of the connection.
pub trait StreamDecryptor {
    /// Decrypts bytes in place.
    fn decrypt(&mut self, data: &mut [u8]) -> Result<(), EncryptionError>;
}
