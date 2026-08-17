//! Entity metadata entries, value types, and terminated list encoding.

use std::{fmt, io::Read};

use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};

use crate::{
    Boolean, Byte, Float, IdOr, PaintingVariant, Position, PrefixedString, ProtocolEnum,
    ResolvableProfile, Slot, TextComponent, TypeCodec, UnsignedByte, VarInt, VarLong,
};

use super::{
    ArmadilloState, BlockStateId, CatSoundVariantId, CatVariantId, ChickenSoundVariantId,
    ChickenVariantId, CopperGolemState, CowSoundVariantId, CowVariantId, Direction, FrogVariantId,
    HumanoidArm, OptionalBlockState, OptionalGlobalPosition, OptionalLivingEntityReference,
    OptionalPosition, OptionalTextComponent, OptionalVarInt, Particle, Particles,
    PigSoundVariantId, PigVariantId, Pose, Quaternion, Rotations, SnifferState, Vector3,
    VillagerData, WeatheringCopperState, WolfSoundVariantId, WolfVariantId,
    ZombieNautilusVariantId,
};

/// Error returned when `0xff` is used as an entity metadata entry index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidEntityMetadataIndex {
    index: u8,
}

impl InvalidEntityMetadataIndex {
    /// Returns the rejected index.
    #[must_use]
    pub const fn index(self) -> u8 {
        self.index
    }
}

impl fmt::Display for InvalidEntityMetadataIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "entity metadata index 0x{:02X} is reserved as the terminator",
            self.index
        )
    }
}

impl std::error::Error for InvalidEntityMetadataIndex {}

/// A valid entity metadata index in the inclusive range `0..=254`.
///
/// Byte value `0xff` cannot be constructed because it terminates the complete
/// [`EntityMetadata`] sequence.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityMetadataIndex(u8);

impl EntityMetadataIndex {
    /// Largest index available to an entry.
    pub const MAX: u8 = 0xfe;
    /// Reserved byte that terminates an entity metadata sequence.
    pub const TERMINATOR: u8 = 0xff;

    /// Creates a valid metadata index.
    pub const fn new(index: u8) -> Result<Self, InvalidEntityMetadataIndex> {
        if index == Self::TERMINATOR {
            Err(InvalidEntityMetadataIndex { index })
        } else {
            Ok(Self(index))
        }
    }

    /// Returns the raw unsigned-byte index.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for EntityMetadataIndex {
    type Error = InvalidEntityMetadataIndex;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EntityMetadataIndex> for u8 {
    fn from(value: EntityMetadataIndex) -> Self {
        value.get()
    }
}

impl TypeCodec for EntityMetadataIndex {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        UnsignedByte(self.0)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataEntry))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let index = UnsignedByte::decode(reader)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataEntry))?
            .0;
        Self::new(index).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::EntityMetadataEntry,
                1,
                InvalidEncodingReason::InvalidEntityMetadataIndex { index },
            )
        })
    }
}

macro_rules! define_metadata_values {
    ($($id:literal => $variant:ident($payload:ty) = $name:literal,)*) => {
        /// Numeric value-type selector used by an entity metadata entry.
        ///
        /// Storing this selector separately from a payload is unnecessary and
        /// could permit mismatches. [`EntityMetadataValue`] is therefore the
        /// primary wire value; this enum is exposed for inspection and registry
        /// mapping, and is returned by [`EntityMetadataValue::value_type`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
        #[protocol_enum(repr = VarInt)]
        pub enum EntityMetadataValueType {
            $(
                #[doc = concat!("The `", $name, "` value type (ID `", stringify!($id), "`).")]
                $variant = $id,
            )*
        }

        /// A metadata type ID bound to exactly the payload required by that type.
        ///
        /// Every variant fixes both the VarInt type selector and its payload,
        /// so an ID/value-layout mismatch cannot be represented in memory.
        #[derive(Debug, Clone, PartialEq)]
        pub enum EntityMetadataValue {
            $(
                #[doc = concat!("A `", $name, "` metadata value.")]
                $variant($payload),
            )*
        }

        impl EntityMetadataValue {
            /// Returns this value's protocol type selector.
            #[must_use]
            pub const fn value_type(&self) -> EntityMetadataValueType {
                match self {
                    $(Self::$variant(_) => EntityMetadataValueType::$variant,)*
                }
            }

            /// Returns the numeric VarInt type ID.
            #[must_use]
            pub fn type_id(&self) -> i32 {
                self.value_type().discriminant() as i32
            }
        }

        impl TypeCodec for EntityMetadataValue {
            fn encode(
                &self,
                writer: &mut impl std::io::Write,
            ) -> Result<(), CodecError> {
                self.value_type()
                    .encode(writer)
                    .map_err(with_value_context)?;
                match self {
                    $(Self::$variant(value) => value.encode(writer).map_err(with_value_context),)*
                }
            }

            fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
                let value_type = EntityMetadataValueType::decode(reader)
                    .map_err(with_value_context)?;
                match value_type {
                    $(
                        EntityMetadataValueType::$variant => <$payload>::decode(reader)
                            .map(Self::$variant)
                            .map_err(with_value_context),
                    )*
                }
            }
        }
    };
}

define_metadata_values! {
    0 => Byte(Byte) = "Byte",
    1 => VarInt(VarInt) = "VarInt",
    2 => VarLong(VarLong) = "VarLong",
    3 => Float(Float) = "Float",
    4 => String(PrefixedString) = "String",
    5 => TextComponent(TextComponent) = "Text Component",
    6 => OptionalTextComponent(OptionalTextComponent) = "Optional Text Component",
    7 => Slot(Slot) = "Slot",
    8 => Boolean(Boolean) = "Boolean",
    9 => Rotations(Rotations) = "Rotations",
    10 => Position(Position) = "Position",
    11 => OptionalPosition(OptionalPosition) = "Optional Position",
    12 => Direction(Direction) = "Direction",
    13 => OptionalLivingEntityReference(OptionalLivingEntityReference) = "Optional Living Entity Reference",
    14 => BlockState(BlockStateId) = "Block State",
    15 => OptionalBlockState(OptionalBlockState) = "Optional Block State",
    16 => Particle(Particle) = "Particle",
    17 => Particles(Particles) = "Particles",
    18 => VillagerData(VillagerData) = "Villager Data",
    19 => OptionalVarInt(OptionalVarInt) = "Optional VarInt",
    20 => Pose(Pose) = "Pose",
    21 => CatVariant(CatVariantId) = "Cat Variant",
    22 => CatSoundVariant(CatSoundVariantId) = "Cat Sound Variant",
    23 => CowVariant(CowVariantId) = "Cow Variant",
    24 => CowSoundVariant(CowSoundVariantId) = "Cow Sound Variant",
    25 => WolfVariant(WolfVariantId) = "Wolf Variant",
    26 => WolfSoundVariant(WolfSoundVariantId) = "Wolf Sound Variant",
    27 => FrogVariant(FrogVariantId) = "Frog Variant",
    28 => PigVariant(PigVariantId) = "Pig Variant",
    29 => PigSoundVariant(PigSoundVariantId) = "Pig Sound Variant",
    30 => ChickenVariant(ChickenVariantId) = "Chicken Variant",
    31 => ChickenSoundVariant(ChickenSoundVariantId) = "Chicken Sound Variant",
    32 => ZombieNautilusVariant(ZombieNautilusVariantId) = "Zombie Nautilus Variant",
    33 => OptionalGlobalPosition(OptionalGlobalPosition) = "Optional Global Position",
    34 => PaintingVariant(IdOr<PaintingVariant>) = "Painting Variant",
    35 => SnifferState(SnifferState) = "Sniffer State",
    36 => ArmadilloState(ArmadilloState) = "Armadillo State",
    37 => CopperGolemState(CopperGolemState) = "Copper Golem State",
    38 => WeatheringCopperState(WeatheringCopperState) = "Weathering Copper State",
    39 => Vector3(Vector3) = "Vector3",
    40 => Quaternion(Quaternion) = "Quaternion",
    41 => ResolvableProfile(ResolvableProfile) = "Resolvable Profile",
    42 => HumanoidArm(HumanoidArm) = "Humanoid Arm",
}

fn with_value_context(error: CodecError) -> CodecError {
    if error.context() == Some(CodecKind::EntityMetadataValue) {
        error
    } else {
        error.with_context(CodecKind::EntityMetadataValue)
    }
}

/// One index and one type-safe value in an entity metadata sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMetadataEntry {
    index: EntityMetadataIndex,
    value: EntityMetadataValue,
}

impl EntityMetadataEntry {
    /// Creates an entry from an already validated index.
    #[must_use]
    pub const fn new(index: EntityMetadataIndex, value: EntityMetadataValue) -> Self {
        Self { index, value }
    }

    /// Creates an entry from a raw index, rejecting the `0xff` terminator.
    pub fn try_new(
        index: u8,
        value: EntityMetadataValue,
    ) -> Result<Self, InvalidEntityMetadataIndex> {
        Ok(Self::new(EntityMetadataIndex::new(index)?, value))
    }

    /// Returns this entry's unique index key.
    #[must_use]
    pub const fn index(&self) -> EntityMetadataIndex {
        self.index
    }

    /// Returns this entry's typed value.
    #[must_use]
    pub const fn value(&self) -> &EntityMetadataValue {
        &self.value
    }

    /// Extracts the typed value.
    #[must_use]
    pub fn into_value(self) -> EntityMetadataValue {
        self.value
    }
}

impl TypeCodec for EntityMetadataEntry {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        self.index.encode(writer)?;
        self.value
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataEntry))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let index = EntityMetadataIndex::decode(reader)?;
        let value = EntityMetadataValue::decode(reader)
            .map_err(|error| error.with_context(CodecKind::EntityMetadataEntry))?;
        Ok(Self { index, value })
    }
}

/// Error returned when an entity metadata sequence repeats an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DuplicateEntityMetadataIndex {
    index: EntityMetadataIndex,
}

impl DuplicateEntityMetadataIndex {
    /// Returns the repeated index.
    #[must_use]
    pub const fn index(self) -> EntityMetadataIndex {
        self.index
    }
}

impl fmt::Display for DuplicateEntityMetadataIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate entity metadata index {}",
            self.index.get()
        )
    }
}

impl std::error::Error for DuplicateEntityMetadataIndex {}

/// A complete `0xff`-terminated entity metadata sequence.
///
/// Entry indices are unique and can only be in `0..=254`; the list codec always
/// appends `0xff` and consumes no bytes after that terminator. Metadata fields
/// may be omitted, so an empty value is encoded as the terminator alone.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     Byte, EntityMetadata, EntityMetadataEntry, EntityMetadataValue, TypeCodec,
/// };
///
/// let metadata = EntityMetadata::new(vec![EntityMetadataEntry::try_new(
///     0,
///     EntityMetadataValue::Byte(Byte(0x20)),
/// )?])?;
///
/// let mut encoded = Vec::new();
/// metadata.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x00, 0x00, 0x20, 0xff]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(EntityMetadata::decode(&mut input)?, metadata);
/// assert!(input.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// See the official [Entity Metadata Format] documentation.
///
/// [Entity Metadata Format]: https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityMetadata {
    entries: Vec<EntityMetadataEntry>,
}

impl EntityMetadata {
    /// Creates a metadata sequence after checking that all indices are unique.
    pub fn new(entries: Vec<EntityMetadataEntry>) -> Result<Self, DuplicateEntityMetadataIndex> {
        validate_unique_indices(&entries)?;
        Ok(Self { entries })
    }

    /// Returns the entries in wire order.
    #[must_use]
    pub fn entries(&self) -> &[EntityMetadataEntry] {
        &self.entries
    }

    /// Extracts the entries in wire order.
    #[must_use]
    pub fn into_entries(self) -> Vec<EntityMetadataEntry> {
        self.entries
    }

    /// Returns the number of metadata entries, excluding the terminator.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether this sequence contains no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the entry at `index`, if present.
    #[must_use]
    pub fn get(&self, index: EntityMetadataIndex) -> Option<&EntityMetadataEntry> {
        self.entries.iter().find(|entry| entry.index == index)
    }

    /// Appends an entry if its index is not already present.
    pub fn push(&mut self, entry: EntityMetadataEntry) -> Result<(), DuplicateEntityMetadataIndex> {
        if self.get(entry.index).is_some() {
            return Err(DuplicateEntityMetadataIndex { index: entry.index });
        }
        self.entries.push(entry);
        Ok(())
    }
}

impl TypeCodec for EntityMetadata {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        if let Err(error) = validate_unique_indices(&self.entries) {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::EntityMetadata,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::DuplicateEntityMetadataIndex {
                    index: error.index.get(),
                },
            ));
        }
        for entry in &self.entries {
            entry
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::EntityMetadata))?;
        }
        UnsignedByte(EntityMetadataIndex::TERMINATOR)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::EntityMetadata))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut entries = Vec::new();
        let mut seen = [false; EntityMetadataIndex::TERMINATOR as usize];
        loop {
            let index = UnsignedByte::decode(reader)
                .map_err(|error| error.with_context(CodecKind::EntityMetadata))?
                .0;
            if index == EntityMetadataIndex::TERMINATOR {
                return Ok(Self { entries });
            }
            if seen[index as usize] {
                return Err(CodecError::invalid_encoding(
                    CodecKind::EntityMetadata,
                    0,
                    InvalidEncodingReason::DuplicateEntityMetadataIndex { index },
                ));
            }
            seen[index as usize] = true;
            let value = EntityMetadataValue::decode(reader)
                .map_err(|error| error.with_context(CodecKind::EntityMetadataEntry))
                .map_err(|error| error.with_context(CodecKind::EntityMetadata))?;
            entries.push(EntityMetadataEntry {
                index: EntityMetadataIndex(index),
                value,
            });
        }
    }
}

fn validate_unique_indices(
    entries: &[EntityMetadataEntry],
) -> Result<(), DuplicateEntityMetadataIndex> {
    let mut seen = [false; EntityMetadataIndex::TERMINATOR as usize];
    for entry in entries {
        let index = entry.index.get() as usize;
        if seen[index] {
            return Err(DuplicateEntityMetadataIndex { index: entry.index });
        }
        seen[index] = true;
    }
    Ok(())
}
