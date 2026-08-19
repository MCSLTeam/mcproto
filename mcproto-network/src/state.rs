//! Negotiated protocol and transform states.

use crate::error::StateError;
use crate::{CompressionCodec, EncryptionError, StreamEncryptor};

/// A Java protocol state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolState {
    /// Initial handshake state.
    Handshaking,
    /// Status query state.
    Status,
    /// Login negotiation state.
    Login,
    /// Configuration state.
    Configuration,
    /// In-game play state.
    Play,
}

/// Packet direction relative to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// From client to server.
    Serverbound,
    /// From server to client.
    Clientbound,
}

/// Compression configuration for one framing direction.
pub enum CompressionMode {
    /// The uncompressed packet format is active.
    Disabled,
    /// The compressed packet format is active.
    Enabled {
        /// Packets at or above this uncompressed size are compressed.
        threshold: usize,
        /// The stateful compression backend.
        codec: Box<dyn CompressionCodec>,
    },
}

impl CompressionMode {
    /// Creates the uncompressed mode.
    #[must_use]
    pub const fn disabled() -> Self {
        Self::Disabled
    }

    /// Creates an enabled mode with a local threshold.
    pub fn enabled(threshold: usize, codec: impl CompressionCodec + 'static) -> Self {
        Self::Enabled {
            threshold,
            codec: Box::new(codec),
        }
    }

    /// Converts the wire value from `Set Compression` into a mode.
    ///
    /// A negative threshold disables compression; a non-negative threshold
    /// enables it. The codec is ignored when the threshold is negative.
    pub fn from_protocol_threshold(
        threshold: i32,
        codec: impl CompressionCodec + 'static,
    ) -> Result<Self, StateError> {
        if threshold < 0 {
            return Ok(Self::Disabled);
        }

        let threshold = usize::try_from(threshold)
            .map_err(|_| StateError::ThresholdOutOfRange { value: threshold })?;
        Ok(Self::enabled(threshold, codec))
    }

    /// Returns the active threshold, if compression is enabled.
    #[must_use]
    pub const fn threshold(&self) -> Option<usize> {
        match self {
            Self::Disabled => None,
            Self::Enabled { threshold, .. } => Some(*threshold),
        }
    }

    /// Returns whether compression is active.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }

    /// Borrows the compression backend when enabled.
    pub fn codec_mut(&mut self) -> Option<&mut dyn CompressionCodec> {
        match self {
            Self::Disabled => None,
            Self::Enabled { codec, .. } => Some(codec.as_mut()),
        }
    }
}

/// Encryption configuration for one stream direction.
pub enum EncryptionMode {
    /// Plaintext transport is active.
    Disabled,
    /// The supplied stateful stream encryptor is active.
    Enabled(Box<dyn StreamEncryptor>),
}

impl EncryptionMode {
    /// Creates the plaintext mode.
    #[must_use]
    pub const fn disabled() -> Self {
        Self::Disabled
    }

    /// Creates an enabled mode from a stateful encryptor.
    pub fn enabled(encryptor: impl StreamEncryptor + 'static) -> Self {
        Self::Enabled(Box::new(encryptor))
    }

    /// Returns whether encryption is active.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }

    /// Borrows the encryptor when enabled.
    pub fn encryptor_mut(&mut self) -> Option<&mut dyn StreamEncryptor> {
        match self {
            Self::Disabled => None,
            Self::Enabled(encryptor) => Some(encryptor.as_mut()),
        }
    }

    /// Applies the encryptor to a frame when enabled.
    pub fn encrypt(&mut self, frame: &mut [u8]) -> Result<(), EncryptionError> {
        if let Some(encryptor) = self.encryptor_mut() {
            encryptor.encrypt(frame)
        } else {
            Ok(())
        }
    }
}
