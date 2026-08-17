//! Structures used by entity metadata value payloads.

use std::{fmt, io::Read};

use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};

use crate::{
    Float, GlobalPosition, Position, PrefixedOptional, ProtocolEnum, TextComponent, TypeCodec,
    TypeStructCodec, Uuid, VarInt,
};

/// Error returned when a numeric registry ID does not fit a non-negative VarInt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidRegistryIdValue {
    value: i64,
}

impl InvalidRegistryIdValue {
    /// Returns the rejected numeric value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.value
    }
}

impl fmt::Display for InvalidRegistryIdValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "registry ID must be between 0 and {}, got {}",
            i32::MAX,
            self.value
        )
    }
}

impl std::error::Error for InvalidRegistryIdValue {}

macro_rules! registry_id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name(u32);

        impl $name {
            /// Greatest registry ID representable by the protocol VarInt.
            pub const MAX: u32 = i32::MAX as u32;

            /// Creates a validated registry ID.
            pub const fn new(value: u32) -> Result<Self, InvalidRegistryIdValue> {
                if value <= Self::MAX {
                    Ok(Self(value))
                } else {
                    Err(InvalidRegistryIdValue {
                        value: value as i64,
                    })
                }
            }

            /// Returns the zero-based numeric registry ID.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }

        impl TryFrom<u32> for $name {
            type Error = InvalidRegistryIdValue;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> Self {
                value.get()
            }
        }

        impl TypeCodec for $name {
            fn encode(
                &self,
                writer: &mut impl std::io::Write,
            ) -> Result<(), CodecError> {
                VarInt(self.0 as i32)
                    .encode(writer)
                    .map_err(|error| error.with_context(CodecKind::RegistryId))
            }

            fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
                let value = VarInt::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::RegistryId))?
                    .0;
                if value < 0 {
                    return Err(CodecError::invalid_encoding(
                        CodecKind::RegistryId,
                        0,
                        InvalidEncodingReason::InvalidRegistryId {
                            value,
                            max: i32::MAX,
                        },
                    ));
                }
                Ok(Self(value as u32))
            }
        }
    };
}

registry_id_type!(/// An ID in the `minecraft:block_state` registry.
    BlockStateId);
registry_id_type!(/// An ID in the `minecraft:cat_variant` registry.
    CatVariantId);
registry_id_type!(/// An ID in the `minecraft:cat_sound_variant` registry.
    CatSoundVariantId);
registry_id_type!(/// An ID in the `minecraft:cow_variant` registry.
    CowVariantId);
registry_id_type!(/// An ID in the `minecraft:cow_sound_variant` registry.
    CowSoundVariantId);
registry_id_type!(/// An ID in the `minecraft:wolf_variant` registry.
    WolfVariantId);
registry_id_type!(/// An ID in the `minecraft:wolf_sound_variant` registry.
    WolfSoundVariantId);
registry_id_type!(/// An ID in the `minecraft:frog_variant` registry.
    FrogVariantId);
registry_id_type!(/// An ID in the `minecraft:pig_variant` registry.
    PigVariantId);
registry_id_type!(/// An ID in the `minecraft:pig_sound_variant` registry.
    PigSoundVariantId);
registry_id_type!(/// An ID in the `minecraft:chicken_variant` registry.
    ChickenVariantId);
registry_id_type!(/// An ID in the `minecraft:chicken_sound_variant` registry.
    ChickenSoundVariantId);
registry_id_type!(/// An ID in the `minecraft:zombie_nautilus_variant` registry.
    ZombieNautilusVariantId);

/// Error returned when constructing a block-state reference that cannot be air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidNonAirBlockStateId {
    /// Registry ID zero denotes air and is reserved as the absent marker.
    Air,
    /// The ID cannot be represented by a non-negative VarInt.
    OutOfRange(u32),
}

impl fmt::Display for InvalidNonAirBlockStateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Air => formatter.write_str(
                "block state ID 0 is air and cannot be represented as a present optional state",
            ),
            Self::OutOfRange(value) => write!(
                formatter,
                "block state ID must be between 1 and {}, got {value}",
                i32::MAX
            ),
        }
    }
}

impl std::error::Error for InvalidNonAirBlockStateId {}

/// A non-air block state usable in the Optional Block State encoding.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NonAirBlockStateId(BlockStateId);

impl NonAirBlockStateId {
    /// Creates a block-state ID in the inclusive range `1..=i32::MAX`.
    pub const fn new(value: u32) -> Result<Self, InvalidNonAirBlockStateId> {
        if value == 0 {
            Err(InvalidNonAirBlockStateId::Air)
        } else if value > i32::MAX as u32 {
            Err(InvalidNonAirBlockStateId::OutOfRange(value))
        } else {
            Ok(Self(BlockStateId(value)))
        }
    }

    /// Returns the zero-based block-state registry ID.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    /// Returns this ID as a regular block-state ID.
    #[must_use]
    pub const fn block_state_id(self) -> BlockStateId {
        self.0
    }
}

/// A block state encoded with zero reserved for absence.
///
/// Unlike a prefixed optional, this value has no Boolean marker. A zero VarInt
/// is absent; every positive VarInt is a non-air block-state registry ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OptionalBlockState {
    /// No block state. Encoded as `VarInt(0)`.
    #[default]
    Absent,
    /// A non-air block state, encoded as its unmodified positive registry ID.
    Present(NonAirBlockStateId),
}

impl TypeCodec for OptionalBlockState {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        let value = match self {
            Self::Absent => 0,
            Self::Present(state) => state.get() as i32,
        };
        VarInt(value)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataValue))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let value = VarInt::decode(reader)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataValue))?
            .0;
        match value {
            0 => Ok(Self::Absent),
            1.. => Ok(Self::Present(NonAirBlockStateId(BlockStateId(
                value as u32,
            )))),
            _ => Err(CodecError::invalid_encoding(
                CodecKind::EntityMetadataValue,
                0,
                InvalidEncodingReason::InvalidRegistryId {
                    value,
                    max: i32::MAX,
                },
            )),
        }
    }
}

/// Error returned for an Optional VarInt value with no unambiguous wire form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidOptionalVarIntValue {
    value: i32,
}

impl InvalidOptionalVarIntValue {
    /// Returns the rejected present value.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

impl fmt::Display for InvalidOptionalVarIntValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Optional VarInt cannot represent present value {}",
            self.value
        )
    }
}

impl std::error::Error for InvalidOptionalVarIntValue {}

/// An optional VarInt encoded as zero for absence and `value + 1` for presence.
///
/// The in-memory value is the actual value, not the incremented wire selector.
/// `-1` collides with the absent selector and `i32::MAX` cannot be incremented,
/// so constructors reject those two present values.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct OptionalVarInt(Option<i32>);

impl OptionalVarInt {
    /// Creates an absent value.
    #[must_use]
    pub const fn none() -> Self {
        Self(None)
    }

    /// Creates a present value when `value + 1` has an unambiguous VarInt form.
    pub const fn some(value: i32) -> Result<Self, InvalidOptionalVarIntValue> {
        if value == -1 || value == i32::MAX {
            Err(InvalidOptionalVarIntValue { value })
        } else {
            Ok(Self(Some(value)))
        }
    }

    /// Returns the actual present value, before wire incrementing.
    #[must_use]
    pub const fn value(self) -> Option<i32> {
        self.0
    }

    /// Returns whether the value is present.
    #[must_use]
    pub const fn is_some(self) -> bool {
        self.0.is_some()
    }

    /// Returns whether the value is absent.
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.0.is_none()
    }
}

impl TryFrom<Option<i32>> for OptionalVarInt {
    type Error = InvalidOptionalVarIntValue;

    fn try_from(value: Option<i32>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => Self::some(value),
            None => Ok(Self::none()),
        }
    }
}

impl TypeCodec for OptionalVarInt {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        let selector = match self.0 {
            None => 0,
            Some(value) => value.checked_add(1).ok_or_else(|| {
                CodecError::invalid_encoding_for_operation(
                    CodecKind::EntityMetadataValue,
                    CodecOperation::Write,
                    0,
                    InvalidEncodingReason::InvalidOptionalVarInt { value },
                )
            })?,
        };
        VarInt(selector)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataValue))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let selector = VarInt::decode(reader)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataValue))?
            .0;
        if selector == 0 {
            return Ok(Self::none());
        }
        let value = selector.checked_sub(1).ok_or_else(|| {
            CodecError::invalid_encoding(
                CodecKind::EntityMetadataValue,
                0,
                InvalidEncodingReason::InvalidOptionalVarInt { value: selector },
            )
        })?;
        Ok(Self(Some(value)))
    }
}

macro_rules! prefixed_optional_metadata_type {
    ($(#[$meta:meta])* $name:ident, $inner:ty) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = EntityMetadataValue)]
        pub struct $name(PrefixedOptional<$inner>);

        impl $name {
            /// Creates a present value.
            #[must_use]
            pub const fn some(value: $inner) -> Self {
                Self(PrefixedOptional::some(value))
            }

            /// Creates an absent value.
            #[must_use]
            pub const fn none() -> Self {
                Self(PrefixedOptional::none())
            }

            /// Returns the contained value by reference, if present.
            #[must_use]
            pub fn value(&self) -> Option<&$inner> {
                self.0.0.0.as_ref()
            }

            /// Extracts the wrapped optional value.
            #[must_use]
            pub fn into_option(self) -> Option<$inner> {
                self.0.into_option()
            }

            /// Returns whether this value is present.
            #[must_use]
            pub const fn is_some(&self) -> bool {
                self.0.is_some()
            }

            /// Returns whether this value is absent.
            #[must_use]
            pub const fn is_none(&self) -> bool {
                self.0.is_none()
            }
        }

        impl From<Option<$inner>> for $name {
            fn from(value: Option<$inner>) -> Self {
                Self(value.into())
            }
        }

        impl From<$name> for Option<$inner> {
            fn from(value: $name) -> Self {
                value.into_option()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::none()
            }
        }
    };
}

prefixed_optional_metadata_type!(/// A Boolean-prefixed optional text component.
    OptionalTextComponent, TextComponent);
prefixed_optional_metadata_type!(/// A Boolean-prefixed optional packed block position.
    OptionalPosition, Position);
prefixed_optional_metadata_type!(/// A Boolean-prefixed optional living-entity UUID reference.
    OptionalLivingEntityReference, Uuid);
prefixed_optional_metadata_type!(/// A Boolean-prefixed optional dimension and block position.
    OptionalGlobalPosition, GlobalPosition);

/// Three Euler rotation components encoded as consecutive Floats.
#[derive(Debug, Clone, Copy, PartialEq, Default, TypeStructCodec)]
#[type_struct_codec(kind = EntityMetadataValue)]
pub struct Rotations {
    /// Rotation around the x axis.
    pub x: Float,
    /// Rotation around the y axis.
    pub y: Float,
    /// Rotation around the z axis.
    pub z: Float,
}

/// A three-dimensional vector encoded as consecutive Floats.
#[derive(Debug, Clone, Copy, PartialEq, Default, TypeStructCodec)]
#[type_struct_codec(kind = EntityMetadataValue)]
pub struct Vector3 {
    /// X component.
    pub x: Float,
    /// Y component.
    pub y: Float,
    /// Z component.
    pub z: Float,
}

/// A quaternion encoded as x, y, z, and w Floats.
#[derive(Debug, Clone, Copy, PartialEq, Default, TypeStructCodec)]
#[type_struct_codec(kind = EntityMetadataValue)]
pub struct Quaternion {
    /// X component.
    pub x: Float,
    /// Y component.
    pub y: Float,
    /// Z component.
    pub z: Float,
    /// W component.
    pub w: Float,
}

/// The six block-face directions used by entity metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

/// An entity pose from the metadata Pose value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum Pose {
    Standing = 0,
    FallFlying = 1,
    Sleeping = 2,
    Swimming = 3,
    SpinAttack = 4,
    Sneaking = 5,
    LongJumping = 6,
    Dying = 7,
    Croaking = 8,
    UsingTongue = 9,
    Sitting = 10,
    Roaring = 11,
    Sniffing = 12,
    Emerging = 13,
    Digging = 14,
    Sliding = 15,
    Shooting = 16,
    Inhaling = 17,
}

/// A built-in villager biome type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum VillagerType {
    Desert = 0,
    Jungle = 1,
    Plains = 2,
    Savanna = 3,
    Snow = 4,
    Swamp = 5,
    Taiga = 6,
}

/// A built-in villager profession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum VillagerProfession {
    None = 0,
    Armorer = 1,
    Butcher = 2,
    Cartographer = 3,
    Cleric = 4,
    Farmer = 5,
    Fisherman = 6,
    Fletcher = 7,
    Leatherworker = 8,
    Librarian = 9,
    Mason = 10,
    Nitwit = 11,
    Shepherd = 12,
    Toolsmith = 13,
    Weaponsmith = 14,
}

/// Villager biome type, profession, and level metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeStructCodec)]
#[type_struct_codec(kind = EntityMetadataValue)]
pub struct VillagerData {
    /// Entry in the `minecraft:villager_type` registry.
    pub villager_type: VillagerType,
    /// Entry in the `minecraft:villager_profession` registry.
    pub profession: VillagerProfession,
    /// Villager trading level.
    pub level: VarInt,
}

/// State used by sniffer entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SnifferState {
    Idling = 0,
    FeelingHappy = 1,
    Scenting = 2,
    Sniffing = 3,
    Searching = 4,
    Digging = 5,
    Rising = 6,
}

/// State used by armadillo entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ArmadilloState {
    Idle = 0,
    Rolling = 1,
    Scared = 2,
    Unrolling = 3,
}

/// State used by copper golem entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum CopperGolemState {
    Idle = 0,
    GettingItem = 1,
    GettingNoItem = 2,
    DroppingItem = 3,
    DroppingNoItem = 4,
}

/// Weathering stage for copper golem metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum WeatheringCopperState {
    Unaffected = 0,
    Exposed = 1,
    Weathered = 2,
    Oxidized = 3,
}

/// Dominant humanoid arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum HumanoidArm {
    Left = 0,
    Right = 1,
}
