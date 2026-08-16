//! Minecraft protocol sound event values.

use crate::{
    TypeCodec,
    basic::{Float, Identifier},
    contextual::PrefixedOptional,
};
use mcproto_codec::error::{CodecError, CodecKind};

/// Describes a sound that can be played.
///
/// A sound event contains its resource [`Identifier`], followed by a boolean
/// indicating whether a fixed range is present, followed by the optional
/// [`Float`] range itself. In memory, the boolean is represented by whether
/// [`fixed_range`](Self::fixed_range) is [`Some`]:
///
/// ```text
/// Identifier + Boolean(has fixed range) + Optional Float
/// ```
///
/// When no fixed range is supplied, playback volume varies with distance
/// according to the sound's normal behavior.
///
/// # Examples
///
/// ```
/// use mcproto_types::{Float, Identifier, SoundEvent, TypeCodec};
///
/// let sound = SoundEvent::fixed(
///     Identifier::new("minecraft:block.note_block.harp")?,
///     Float(16.0),
/// );
/// let mut encoded = Vec::new();
/// sound.encode(&mut encoded)?;
///
/// let mut input = encoded.as_slice();
/// assert_eq!(SoundEvent::decode(&mut input)?, sound);
/// assert!(input.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SoundEvent {
    /// The sound's resource identifier.
    pub sound_name: Identifier,
    /// Maximum playback range, or `None` for distance-dependent volume.
    pub fixed_range: Option<Float>,
}

impl SoundEvent {
    /// Creates a sound event with an optional fixed range.
    #[must_use]
    pub const fn new(sound_name: Identifier, fixed_range: Option<Float>) -> Self {
        Self {
            sound_name,
            fixed_range,
        }
    }

    /// Creates a sound event with distance-dependent volume.
    #[must_use]
    pub const fn variable(sound_name: Identifier) -> Self {
        Self::new(sound_name, None)
    }

    /// Creates a sound event with a fixed maximum range.
    #[must_use]
    pub const fn fixed(sound_name: Identifier, fixed_range: Float) -> Self {
        Self::new(sound_name, Some(fixed_range))
    }

    /// Returns whether this sound has a fixed maximum range.
    #[must_use]
    pub const fn has_fixed_range(&self) -> bool {
        self.fixed_range.is_some()
    }
}

impl TypeCodec for SoundEvent {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        self.sound_name
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::SoundEvent))?;
        PrefixedOptional::from(self.fixed_range)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::SoundEvent))
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let sound_name = Identifier::decode(reader)
            .map_err(|error| error.with_context(CodecKind::SoundEvent))?;
        let fixed_range = PrefixedOptional::<Float>::decode(reader)
            .map_err(|error| error.with_context(CodecKind::SoundEvent))?
            .into_option();
        Ok(Self::new(sound_name, fixed_range))
    }
}
