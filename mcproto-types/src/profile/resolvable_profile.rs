//! Profiles that may contain partial identity data and texture overrides.

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, InvalidEncodingReason},
    varint::{VarIntRead, VarIntWrite},
};

use crate::{
    GameProfile, GameProfileProperties, GameProfileUsername, Identifier, PrefixedOptional,
    ProtocolEnum, TypeCodec, TypeStructCodec, Uuid, VarInt,
};

/// The unresolved subset of a game profile.
///
/// Username and UUID each carry their own boolean presence marker. Properties
/// use the same bounded representation as [`GameProfile::properties`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = PartialProfile)]
pub struct PartialProfile {
    /// Optional player username, limited to 16 UTF-16 code units.
    pub username: PrefixedOptional<GameProfileUsername>,
    /// Optional player UUID.
    pub uuid: PrefixedOptional<Uuid>,
    /// Signed or unsigned profile properties, limited to 16 entries.
    pub properties: GameProfileProperties,
}

impl Default for PartialProfile {
    fn default() -> Self {
        Self {
            username: PrefixedOptional::none(),
            uuid: PrefixedOptional::none(),
            properties: GameProfileProperties::default(),
        }
    }
}

/// Player skin model used by a resolvable profile override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SkinModel {
    /// The standard, wide-arm player model.
    #[default]
    Wide = 0,
    /// The slim-arm player model.
    Slim = 1,
}

/// Identity data carried by a [`ResolvableProfile`].
///
/// The wire discriminant is a VarInt: `0` selects [`Partial`](Self::Partial)
/// and `1` selects [`Complete`](Self::Complete).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolvableProfileData {
    /// Identity data that may omit the username or UUID.
    Partial(PartialProfile),
    /// A fully specified [`GameProfile`].
    Complete(GameProfile),
}

impl From<PartialProfile> for ResolvableProfileData {
    fn from(value: PartialProfile) -> Self {
        Self::Partial(value)
    }
}

impl From<GameProfile> for ResolvableProfileData {
    fn from(value: GameProfile) -> Self {
        Self::Complete(value)
    }
}

/// A partial or complete game profile followed by optional skin overrides.
///
/// The profile starts with a VarInt kind (`0` for partial, `1` for complete).
/// Partial data contains a prefixed optional username, a prefixed optional
/// UUID, and at most 16 properties. It is followed by optional body, cape,
/// elytra, and model overrides. Each override carries a boolean presence
/// marker in the packet codec.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     GameProfileUsername, Identifier, PartialProfile, PrefixedOptional,
///     ResolvableProfile, ResolvableProfileData, SkinModel, TypeCodec,
/// };
///
/// let mut profile = ResolvableProfile::new(ResolvableProfileData::Partial(
///     PartialProfile {
///         username: PrefixedOptional::some(GameProfileUsername::new("Alex")?),
///         ..PartialProfile::default()
///     },
/// ));
/// profile.body = PrefixedOptional::some(Identifier::new("minecraft:entity/player/wide/alex")?);
/// profile.model = PrefixedOptional::some(SkinModel::Wide);
///
/// let mut encoded = Vec::new();
/// profile.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(ResolvableProfile::decode(&mut input)?, profile);
/// assert!(input.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// See the official [Resolvable Profile] protocol documentation.
///
/// [Resolvable Profile]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resolvable_Profile
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvableProfile {
    /// Partial or complete identity data.
    pub profile: ResolvableProfileData,
    /// Optional override for the body texture.
    pub body: PrefixedOptional<Identifier>,
    /// Optional override for the cape texture.
    pub cape: PrefixedOptional<Identifier>,
    /// Optional override for the elytra texture.
    pub elytra: PrefixedOptional<Identifier>,
    /// Optional override for the player model.
    pub model: PrefixedOptional<SkinModel>,
}

impl ResolvableProfile {
    /// Creates a profile without texture or model overrides.
    #[must_use]
    pub const fn new(profile: ResolvableProfileData) -> Self {
        Self {
            profile,
            body: PrefixedOptional::none(),
            cape: PrefixedOptional::none(),
            elytra: PrefixedOptional::none(),
            model: PrefixedOptional::none(),
        }
    }

    /// Creates a resolvable profile from partial identity data.
    #[must_use]
    pub const fn partial(profile: PartialProfile) -> Self {
        Self::new(ResolvableProfileData::Partial(profile))
    }

    /// Creates a resolvable profile from a complete game profile.
    #[must_use]
    pub const fn complete(profile: GameProfile) -> Self {
        Self::new(ResolvableProfileData::Complete(profile))
    }
}

impl From<PartialProfile> for ResolvableProfile {
    fn from(value: PartialProfile) -> Self {
        Self::partial(value)
    }
}

impl From<GameProfile> for ResolvableProfile {
    fn from(value: GameProfile) -> Self {
        Self::complete(value)
    }
}

impl TypeCodec for ResolvableProfile {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        match &self.profile {
            ResolvableProfileData::Partial(value) => {
                writer
                    .write_varint(0)
                    .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
                value
                    .encode(writer)
                    .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
            }
            ResolvableProfileData::Complete(value) => {
                writer
                    .write_varint(1)
                    .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
                value
                    .encode(writer)
                    .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
            }
        }

        self.body
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
        self.cape
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
        self.elytra
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
        self.model
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ResolvableProfile))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (kind, kind_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?;
        let profile = match kind {
            0 => PartialProfile::decode(reader)
                .map(ResolvableProfileData::Partial)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
            1 => GameProfile::decode(reader)
                .map(ResolvableProfileData::Complete)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
            value => {
                return Err(CodecError::invalid_encoding(
                    CodecKind::ResolvableProfile,
                    kind_size,
                    InvalidEncodingReason::InvalidEnumValue {
                        value: i128::from(value),
                    },
                ));
            }
        };

        Ok(Self {
            profile,
            body: PrefixedOptional::decode(reader)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
            cape: PrefixedOptional::decode(reader)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
            elytra: PrefixedOptional::decode(reader)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
            model: PrefixedOptional::decode(reader)
                .map_err(|error| error.with_context(CodecKind::ResolvableProfile))?,
        })
    }
}
