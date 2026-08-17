//! Minecraft player game profiles.

use std::{fmt, io::Read};

use mcproto_codec::{
    error::{CodecError, CodecKind, InvalidEncodingReason},
    varint::{VarIntRead, VarIntWrite},
};

use crate::{
    TypeCodec, TypeStructCodec,
    basic::{Uuid, decode_prefixed_string, encode_prefixed_string},
    contextual::PrefixedOptional,
};

/// Maximum number of properties in a game profile.
pub const MAX_GAME_PROFILE_PROPERTIES: usize = 16;

/// A protocol string with a game-profile-specific UTF-16 length limit.
///
/// Use the aliases [`GameProfileUsername`], [`GameProfilePropertyName`],
/// [`GameProfilePropertyValue`], and [`GameProfilePropertySignature`] rather
/// than spelling the const generic directly.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GameProfileString<const MAX_UTF16_CODE_UNITS: usize>(String);

impl<const MAX_UTF16_CODE_UNITS: usize> GameProfileString<MAX_UTF16_CODE_UNITS> {
    /// Maximum number of UTF-16 code units accepted by this string type.
    pub const MAX_UTF16_CODE_UNITS: usize = MAX_UTF16_CODE_UNITS;
    /// Maximum UTF-8 payload size accepted by this string type.
    pub const MAX_BYTES: usize = MAX_UTF16_CODE_UNITS.saturating_mul(3);

    /// Creates a string after checking its UTF-16 code-unit limit.
    pub fn new(value: impl Into<String>) -> Result<Self, GameProfileStringTooLong> {
        let value = value.into();
        let actual_code_units = value.encode_utf16().count();
        if actual_code_units > MAX_UTF16_CODE_UNITS {
            return Err(GameProfileStringTooLong {
                max_code_units: MAX_UTF16_CODE_UNITS,
                actual_code_units,
            });
        }
        Ok(Self(value))
    }

    /// Returns the string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extracts the owned string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> AsRef<str> for GameProfileString<MAX_UTF16_CODE_UNITS> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> fmt::Display for GameProfileString<MAX_UTF16_CODE_UNITS> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> TryFrom<String>
    for GameProfileString<MAX_UTF16_CODE_UNITS>
{
    type Error = GameProfileStringTooLong;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> TryFrom<&str> for GameProfileString<MAX_UTF16_CODE_UNITS> {
    type Error = GameProfileStringTooLong;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> From<GameProfileString<MAX_UTF16_CODE_UNITS>> for String {
    fn from(value: GameProfileString<MAX_UTF16_CODE_UNITS>) -> Self {
        value.into_inner()
    }
}

impl<const MAX_UTF16_CODE_UNITS: usize> TypeCodec for GameProfileString<MAX_UTF16_CODE_UNITS> {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        encode_prefixed_string(
            &self.0,
            writer,
            CodecKind::String,
            Self::MAX_BYTES,
            MAX_UTF16_CODE_UNITS,
        )
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        decode_prefixed_string(
            reader,
            CodecKind::String,
            Self::MAX_BYTES,
            MAX_UTF16_CODE_UNITS,
        )
        .map(|(value, _)| Self(value))
    }
}

/// Error returned when a game-profile string exceeds its field limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameProfileStringTooLong {
    /// Maximum permitted number of UTF-16 code units.
    pub max_code_units: usize,
    /// Number of UTF-16 code units in the rejected value.
    pub actual_code_units: usize,
}

impl fmt::Display for GameProfileStringTooLong {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "game-profile string contains {} UTF-16 code units; maximum is {}",
            self.actual_code_units, self.max_code_units
        )
    }
}

impl std::error::Error for GameProfileStringTooLong {}

/// A game profile username, limited to 16 UTF-16 code units.
pub type GameProfileUsername = GameProfileString<16>;
/// A game profile property name, limited to 64 UTF-16 code units.
pub type GameProfilePropertyName = GameProfileString<64>;
/// A game profile property value, limited to 32,767 UTF-16 code units.
pub type GameProfilePropertyValue = GameProfileString<32767>;
/// A game profile property signature, limited to 1024 UTF-16 code units.
pub type GameProfilePropertySignature = GameProfileString<1024>;

/// One property attached to a [`GameProfile`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = GameProfileProperty)]
pub struct GameProfileProperty {
    /// Property name, commonly `textures`.
    pub name: GameProfilePropertyName,
    /// Property value, commonly Base64-encoded JSON.
    pub value: GameProfilePropertyValue,
    /// Optional cryptographic signature for the property value.
    pub signature: PrefixedOptional<GameProfilePropertySignature>,
}

impl GameProfileProperty {
    /// Creates an unsigned game profile property.
    #[must_use]
    pub const fn new(name: GameProfilePropertyName, value: GameProfilePropertyValue) -> Self {
        Self {
            name,
            value,
            signature: PrefixedOptional::none(),
        }
    }
}

/// A length-prefixed game profile property list containing at most 16 entries.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GameProfileProperties(Vec<GameProfileProperty>);

impl GameProfileProperties {
    /// Maximum number of properties permitted by the protocol.
    pub const MAX_LEN: usize = MAX_GAME_PROFILE_PROPERTIES;

    /// Creates a property list after enforcing the 16-entry limit.
    pub fn new(properties: Vec<GameProfileProperty>) -> Result<Self, TooManyGameProfileProperties> {
        if properties.len() > Self::MAX_LEN {
            return Err(TooManyGameProfileProperties {
                actual: properties.len(),
            });
        }
        Ok(Self(properties))
    }

    /// Returns the number of properties.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the property list is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the properties as a slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[GameProfileProperty] {
        self.0.as_slice()
    }

    /// Extracts the property vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<GameProfileProperty> {
        self.0
    }

    /// Adds one property if the protocol limit has not been reached.
    pub fn push(
        &mut self,
        property: GameProfileProperty,
    ) -> Result<(), TooManyGameProfileProperties> {
        if self.len() == Self::MAX_LEN {
            return Err(TooManyGameProfileProperties {
                actual: self.len() + 1,
            });
        }
        self.0.push(property);
        Ok(())
    }
}

impl TryFrom<Vec<GameProfileProperty>> for GameProfileProperties {
    type Error = TooManyGameProfileProperties;

    fn try_from(value: Vec<GameProfileProperty>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GameProfileProperties> for Vec<GameProfileProperty> {
    fn from(value: GameProfileProperties) -> Self {
        value.into_vec()
    }
}

impl TypeCodec for GameProfileProperties {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        writer
            .write_varint(self.len() as i32)
            .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        for property in &self.0 {
            property
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        if length < 0 {
            return Err(CodecError::invalid_encoding(
                CodecKind::PrefixedArray,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: length },
            ));
        }
        let length = length as usize;
        if length > Self::MAX_LEN {
            return Err(CodecError::invalid_encoding(
                CodecKind::PrefixedArray,
                prefix_size,
                InvalidEncodingReason::LengthOutOfRange {
                    max: Self::MAX_LEN,
                    actual: length,
                },
            ));
        }

        let mut properties = Vec::with_capacity(length);
        for _ in 0..length {
            properties.push(
                GameProfileProperty::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::PrefixedArray))?,
            );
        }
        Ok(Self(properties))
    }
}

/// Error returned when a game profile contains more than 16 properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TooManyGameProfileProperties {
    /// Number of properties in the rejected list.
    pub actual: usize,
}

impl fmt::Display for TooManyGameProfileProperties {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "game profile contains {} properties; maximum is {}",
            self.actual, MAX_GAME_PROFILE_PROPERTIES
        )
    }
}

impl std::error::Error for TooManyGameProfileProperties {}

/// A Minecraft player profile.
///
/// The protocol encodes a UUID, a username limited to 16 UTF-16 code units,
/// and a VarInt-prefixed list of at most 16 [`GameProfileProperty`] values.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     GameProfile, GameProfileProperty, GameProfilePropertyName,
///     GameProfilePropertyValue, GameProfileUsername, TypeCodec, Uuid,
/// };
///
/// let mut profile = GameProfile::new(
///     Uuid::from_bytes([0; 16]),
///     GameProfileUsername::new("Player")?,
/// );
/// profile.properties.push(GameProfileProperty::new(
///     GameProfilePropertyName::new("textures")?,
///     GameProfilePropertyValue::new("base64-value")?,
/// ))?;
///
/// let mut encoded = Vec::new();
/// profile.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(GameProfile::decode(&mut input)?, profile);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// See the official [Game Profile] protocol documentation.
///
/// [Game Profile]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Game_Profile
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = GameProfile)]
pub struct GameProfile {
    /// Universally unique player identifier.
    pub uuid: Uuid,
    /// Player username, limited to 16 UTF-16 code units.
    pub username: GameProfileUsername,
    /// Signed or unsigned player properties, limited to 16 entries.
    pub properties: GameProfileProperties,
}

impl GameProfile {
    /// Creates a game profile with no properties.
    #[must_use]
    pub const fn new(uuid: Uuid, username: GameProfileUsername) -> Self {
        Self {
            uuid,
            username,
            properties: GameProfileProperties(Vec::new()),
        }
    }
}
