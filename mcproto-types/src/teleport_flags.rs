//! Minecraft protocol teleport flags.

use mcproto_codec::error::{CodecError, CodecKind};

use crate::{TypeCodec, basic::Int};

bitflags::bitflags! {
    /// Specifies how teleportation is applied to position, rotation, and velocity.
    ///
    /// Teleport flags are represented on the wire as a four-byte [`Int`]. For
    /// each of the lower eight bits, a set bit makes the corresponding value
    /// relative and an unset bit makes it absolute. Bit `0x0100` additionally
    /// rotates velocity by the teleport's change in rotation before applying
    /// the velocity change.
    ///
    /// Unknown bits are retained when decoding so values from newer protocol
    /// versions can be decoded and re-encoded without losing information.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcproto_types::{TeleportFlags, TypeCodec};
    ///
    /// let flags = TeleportFlags::RELATIVE_X
    ///     | TeleportFlags::RELATIVE_YAW
    ///     | TeleportFlags::ROTATE_VELOCITY;
    /// let mut encoded = Vec::new();
    /// flags.encode(&mut encoded)?;
    /// assert_eq!(encoded, [0x00, 0x00, 0x01, 0x09]);
    ///
    /// let mut input = encoded.as_slice();
    /// assert_eq!(TeleportFlags::decode(&mut input)?, flags);
    /// assert!(input.is_empty());
    /// # Ok::<(), mcproto_codec::error::CodecError>(())
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TeleportFlags: u32 {
        /// Apply the X position relatively.
        const RELATIVE_X = 0x0001;
        /// Apply the Y position relatively.
        const RELATIVE_Y = 0x0002;
        /// Apply the Z position relatively.
        const RELATIVE_Z = 0x0004;
        /// Apply yaw relatively.
        const RELATIVE_YAW = 0x0008;
        /// Apply pitch relatively.
        const RELATIVE_PITCH = 0x0010;
        /// Apply X velocity relatively.
        const RELATIVE_VELOCITY_X = 0x0020;
        /// Apply Y velocity relatively.
        const RELATIVE_VELOCITY_Y = 0x0040;
        /// Apply Z velocity relatively.
        const RELATIVE_VELOCITY_Z = 0x0080;
        /// Rotate velocity by the change in rotation before applying its change.
        const ROTATE_VELOCITY = 0x0100;
    }
}

impl TypeCodec for TeleportFlags {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        Int(self.bits() as i32)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::TeleportFlags))
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let bits = Int::decode(reader)
            .map_err(|error| error.with_context(CodecKind::TeleportFlags))?
            .0 as u32;
        Ok(Self::from_bits_retain(bits))
    }
}
