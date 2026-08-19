//! Errors produced by the network framing layer.

use std::{error::Error, fmt, io};

use mcproto_codec::error::CodecError;
use thiserror::Error;

/// A boxed backend error retained as the source of a network-layer error.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Broad, stable classification of a network error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The peer closed the stream at a packet boundary.
    Closed,
    /// The transport returned an I/O error.
    Transport,
    /// A packet frame was malformed or exceeded a configured limit.
    Frame,
    /// A packet body failed its `TypeCodec` implementation.
    PacketCodec,
    /// Compression or decompression failed.
    Compression,
    /// Encryption setup or stream processing failed.
    Encryption,
    /// A local connection-state transition was invalid.
    State,
}

/// The transport operation in which an I/O error occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoOperation {
    /// Reading bytes from the transport.
    Read,
    /// Writing bytes to the transport.
    Write,
    /// Flushing buffered bytes.
    Flush,
}

impl fmt::Display for IoOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Read => "reading",
            Self::Write => "writing",
            Self::Flush => "flushing",
        })
    }
}

/// An error returned by the public network API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NetworkError {
    /// A transport-level I/O failure.
    #[error("I/O error while {operation}: {source}")]
    Transport {
        /// The operation that failed.
        operation: IoOperation,
        /// The underlying transport error.
        #[source]
        source: io::Error,
    },

    /// The peer closed the connection between packet frames.
    #[error("peer closed the connection")]
    Closed,

    /// The peer closed the connection after a frame had started.
    #[error("truncated packet frame: expected {expected} bytes, received {received}")]
    TruncatedFrame {
        /// Number of frame bytes expected after the length prefix.
        expected: usize,
        /// Number of frame bytes received.
        received: usize,
    },

    /// A packet frame violated the protocol or configured limits.
    #[error("invalid packet frame: {0}")]
    Frame(#[from] FrameError),

    /// A packet body's type codec failed.
    #[error("packet codec failed for packet {packet_id:?}: {source}")]
    PacketCodec {
        /// The packet ID when it was available.
        packet_id: Option<i32>,
        /// The detailed codec error.
        #[source]
        source: CodecError,
    },

    /// A compression backend or compressed payload failed.
    #[error("compression failed: {0}")]
    Compression(#[from] CompressionError),

    /// An encryption backend or encrypted stream failed.
    #[error("encryption failed: {0}")]
    Encryption(#[from] EncryptionError),

    /// A local state transition was invalid.
    #[error("invalid connection state: {0}")]
    State(#[from] StateError),
}

impl NetworkError {
    /// Returns a stable high-level classification for this error.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        match self {
            Self::Closed => ErrorKind::Closed,
            Self::Transport { .. } => ErrorKind::Transport,
            Self::TruncatedFrame { .. } | Self::Frame(_) => ErrorKind::Frame,
            Self::PacketCodec { .. } => ErrorKind::PacketCodec,
            Self::Compression(_) => ErrorKind::Compression,
            Self::Encryption(_) => ErrorKind::Encryption,
            Self::State(_) => ErrorKind::State,
        }
    }

    /// Creates a transport error while retaining the original I/O source.
    #[must_use]
    pub const fn transport(operation: IoOperation, source: io::Error) -> Self {
        Self::Transport { operation, source }
    }
}

/// Errors specific to a packet frame.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// The outer packet length was negative.
    #[error("packet length is negative: {value}")]
    NegativeLength {
        /// The value decoded from the wire.
        value: i32,
    },

    /// A length exceeded the configured or representable maximum.
    #[error("packet length {actual} exceeds maximum {maximum}")]
    LengthTooLarge {
        /// The observed length.
        actual: usize,
        /// The permitted maximum.
        maximum: usize,
    },

    /// The compressed payload's decoded length differed from its declaration.
    #[error("decompressed length mismatch: declared {declared}, decoded {actual}")]
    DecompressedLengthMismatch {
        /// The Data Length field.
        declared: usize,
        /// The actual decompressed length.
        actual: usize,
    },

    /// An uncompressed packet was sent when it should have crossed the threshold.
    #[error("packet is not compressed despite meeting threshold {threshold}")]
    BelowCompressionThreshold {
        /// The active compression threshold.
        threshold: usize,
    },

    /// The packet payload did not contain a packet ID.
    #[error("packet payload does not contain a packet ID")]
    MissingPacketId,
}

/// Errors reported by a compression implementation.
#[derive(Debug, Error)]
pub enum CompressionError {
    /// The backend rejected or failed to process the data.
    #[error("compression backend error: {source}")]
    Backend {
        /// The backend's original error.
        #[source]
        source: BoxError,
    },

    /// The decoded output did not have the protocol-declared size.
    #[error("decompressed length mismatch: declared {declared}, decoded {actual}")]
    LengthMismatch {
        /// The Data Length field.
        declared: usize,
        /// The actual output length.
        actual: usize,
    },

    /// The output exceeded the configured limit.
    #[error("decompressed output {actual} exceeds maximum {maximum}")]
    OutputTooLarge {
        /// The output size.
        actual: usize,
        /// The configured maximum.
        maximum: usize,
    },
}

/// Errors reported by an encryption implementation.
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// The backend rejected or failed to process the data.
    #[error("encryption backend error: {source}")]
    Backend {
        /// The backend's original error.
        #[source]
        source: BoxError,
    },

    /// A transition was attempted while plaintext is still buffered.
    #[error("cannot enable encryption while plaintext is pending")]
    PendingPlaintext,
}

/// Errors caused by an invalid local connection-state transition.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum StateError {
    /// A compression mode was enabled twice.
    #[error("compression is already enabled")]
    CompressionAlreadyEnabled,

    /// An encryption mode was enabled twice.
    #[error("encryption is already enabled")]
    EncryptionAlreadyEnabled,

    /// A protocol threshold could not be represented on this platform.
    #[error("compression threshold cannot be represented: {value}")]
    ThresholdOutOfRange {
        /// The wire value.
        value: i32,
    },
}

/// Errors returned when creating a packet ID.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("packet ID must be non-negative, got {value}")]
pub struct PacketIdError {
    /// The rejected value.
    pub value: i32,
}
