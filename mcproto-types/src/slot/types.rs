//! Structures shared by multiple structured item components.

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
};

use crate::{
    Boolean, Double, Float, IdOr, IdSet, Identifier, Int, Nbt, Position, PrefixedArray,
    PrefixedOptional, PrefixedString, ProtocolEnum, SoundEvent, TextComponent, TypeCodec,
    TypeStructCodec, Uuid, VarInt,
};

use super::DataComponent;

/// A network NBT `TAG_String` root value.
///
/// Unlike [`Nbt`], which represents a root compound, this type writes tag ID
/// `8` followed directly by the NBT string payload. NBT uses Java CESU-8.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NbtString(pub String);

impl TypeCodec for NbtString {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let encoded = cesu8::to_java_cesu8(&self.0);
        let length = u16::try_from(encoded.len()).map_err(|_| {
            CodecError::invalid_encoding_for_operation(
                CodecKind::Nbt,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::TooLong {
                    max_bytes: u16::MAX as usize,
                },
            )
        })?;
        write_all_counted(writer, &[8], CodecKind::Nbt, 0)?;
        write_all_counted(writer, &length.to_be_bytes(), CodecKind::Nbt, 1)?;
        write_all_counted(writer, encoded.as_ref(), CodecKind::Nbt, 3)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let mut tag = [0; 1];
        read_exact_counted(reader, &mut tag, CodecKind::Nbt, 0)?;
        if tag[0] != 8 {
            return Err(CodecError::invalid_encoding(
                CodecKind::Nbt,
                1,
                InvalidEncodingReason::InvalidNbt,
            ));
        }
        let mut length = [0; 2];
        read_exact_counted(reader, &mut length, CodecKind::Nbt, 1)?;
        let mut encoded = vec![0; u16::from_be_bytes(length) as usize];
        read_exact_counted(reader, &mut encoded, CodecKind::Nbt, 3)?;
        let value = cesu8::from_java_cesu8(&encoded).map_err(|_| {
            CodecError::invalid_encoding(
                CodecKind::Nbt,
                3 + encoded.len(),
                InvalidEncodingReason::InvalidNbt,
            )
        })?;
        Ok(Self(value.into_owned()))
    }
}

macro_rules! protocol_struct {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = StructuredComponent)]
        pub struct $name { $($(#[$field_meta])* pub $field: $ty,)* }
    };
}

/// A raw value and an optional filtered replacement.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = StructuredComponent)]
pub struct Filterable<T> {
    /// Unfiltered value.
    pub raw: T,
    /// Filtered value, when supplied.
    pub filtered: PrefixedOptional<T>,
}

protocol_struct!(/// One enchantment and its level.
    EnchantmentEntry { type_id: VarInt, level: VarInt });

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum Rarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum AttributeOperation {
    Add = 0,
    MultiplyBase = 1,
    MultiplyTotal = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum AttributeSlot {
    Any = 0,
    MainHand = 1,
    OffHand = 2,
    Hand = 3,
    Feet = 4,
    Legs = 5,
    Chest = 6,
    Head = 7,
    Armor = 8,
    Body = 9,
}

protocol_struct!(/// One item attribute modifier.
AttributeModifier {
    attribute_id: VarInt,
    modifier_id: Identifier,
    value: Double,
    operation: AttributeOperation,
    slot: AttributeSlot,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ItemUseAnimation {
    None = 0,
    Eat = 1,
    Drink = 2,
    Block = 3,
    Bow = 4,
    Spear = 5,
    Crossbow = 6,
    Spyglass = 7,
    TootHorn = 8,
    Brush = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum EquipmentSlot {
    MainHand = 0,
    Feet = 1,
    Legs = 2,
    Chest = 3,
    Head = 4,
    OffHand = 5,
    Body = 6,
}

protocol_struct!(/// A tool rule for a set of blocks.
ToolRule {
    blocks: IdSet,
    speed: PrefixedOptional<Float>,
    correct_drop_for_blocks: PrefixedOptional<Boolean>,
});

protocol_struct!(/// One damage reduction rule for a blocking item.
DamageReduction {
    horizontal_blocking_angle: Float,
    damage_types: PrefixedOptional<IdSet>,
    base: Float,
    factor: Float,
});

protocol_struct!(/// A condition used by a kinetic weapon action.
KineticWeaponCondition {
    max_duration_ticks: VarInt,
    min_speed: Float,
    min_relative_speed: Float,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SwingAnimationType {
    None = 0,
    Whack = 1,
    Stab = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum MapPostProcessing {
    Lock = 0,
    Scale = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum DyeColor {
    White = 0,
    Orange = 1,
    Magenta = 2,
    LightBlue = 3,
    Yellow = 4,
    Lime = 5,
    Pink = 6,
    Gray = 7,
    LightGray = 8,
    Cyan = 9,
    Purple = 10,
    Blue = 11,
    Brown = 12,
    Green = 13,
    Red = 14,
    Black = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum FireworkShape {
    SmallBall = 0,
    LargeBall = 1,
    Star = 2,
    Creeper = 3,
    Burst = 4,
}

protocol_struct!(/// A firework explosion description.
FireworkExplosion {
    shape: FireworkShape,
    colors: PrefixedArray<Int>,
    fade_colors: PrefixedArray<Int>,
    has_trail: Boolean,
    has_twinkle: Boolean,
});

protocol_struct!(/// A suspicious stew effect entry.
    SuspiciousStewEffect { type_id: VarInt, duration: VarInt });

protocol_struct!(/// A potion effect and its recursive details.
    PotionEffect { type_id: VarInt, details: PotionEffectDetail });

/// Detailed potion effect settings, including an optional hidden effect.
#[derive(Debug, Clone, PartialEq)]
pub struct PotionEffectDetail {
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub ambient: Boolean,
    pub show_particles: Boolean,
    pub show_icon: Boolean,
    pub hidden_effect: PrefixedOptional<Box<PotionEffectDetail>>,
}

impl TypeCodec for PotionEffectDetail {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.amplifier.encode(writer)?;
        self.duration.encode(writer)?;
        self.ambient.encode(writer)?;
        self.show_particles.encode(writer)?;
        self.show_icon.encode(writer)?;
        self.hidden_effect.encode(writer)
    }
    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            amplifier: VarInt::decode(reader)?,
            duration: VarInt::decode(reader)?,
            ambient: Boolean::decode(reader)?,
            show_particles: Boolean::decode(reader)?,
            show_icon: Boolean::decode(reader)?,
            hidden_effect: PrefixedOptional::decode(reader)?,
        })
    }
}

protocol_struct!(/// An armor material asset override.
    TrimMaterialOverride { armor_material: Identifier, asset_name: PrefixedString });
protocol_struct!(/// Inline trim material data.
TrimMaterial {
    suffix: PrefixedString,
    overrides: PrefixedArray<TrimMaterialOverride>,
    description: TextComponent,
});
protocol_struct!(/// Inline trim pattern data.
TrimPattern {
    asset_name: PrefixedString,
    template_item: VarInt,
    description: TextComponent,
    decal: Boolean,
});
protocol_struct!(/// Inline instrument data.
Instrument {
    sound_event: IdOr<SoundEvent>,
    use_duration: Float,
    range: Float,
    description: TextComponent,
});
protocol_struct!(/// Inline jukebox song data.
JukeboxSong {
    sound_event: IdOr<SoundEvent>,
    description: TextComponent,
    duration: Float,
    output: VarInt,
});
protocol_struct!(/// Inline banner pattern data.
    BannerPattern { asset_id: Identifier, translation_key: PrefixedString });
protocol_struct!(/// One banner pattern layer.
    BannerPatternLayer { pattern: IdOr<BannerPattern>, color: DyeColor });
protocol_struct!(/// One block-state name/value pair.
    BlockStateProperty { name: PrefixedString, value: PrefixedString });
protocol_struct!(/// A bee stored in a hive.
Bee {
    entity_type_id: VarInt,
    entity_data: Nbt,
    ticks_in_hive: VarInt,
    min_ticks_in_hive: VarInt,
});
protocol_struct!(/// A dimension and block position.
    GlobalPosition { dimension: Identifier, position: Position });
protocol_struct!(/// A game profile property used by Slot components.
    SlotProfileProperty {
    name: PrefixedString,
    value: PrefixedString,
    signature: PrefixedOptional<PrefixedString>,
});
protocol_struct!(/// A complete game profile.
GameProfile {
    uuid: Uuid,
    username: PrefixedString,
        properties: PrefixedArray<SlotProfileProperty>,
});
protocol_struct!(/// A partial game profile.
PartialProfile {
    username: PrefixedOptional<PrefixedString>,
    uuid: PrefixedOptional<Uuid>,
        properties: PrefixedArray<SlotProfileProperty>,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SkinModel {
    Wide = 0,
    Slim = 1,
}

/// Profile data followed by optional texture overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvableProfile {
    pub profile: ResolvableProfileData,
    pub body: PrefixedOptional<Identifier>,
    pub cape: PrefixedOptional<Identifier>,
    pub elytra: PrefixedOptional<Identifier>,
    pub model: PrefixedOptional<SkinModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvableProfileData {
    Partial(PartialProfile),
    Complete(GameProfile),
}

impl TypeCodec for ResolvableProfile {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        match &self.profile {
            ResolvableProfileData::Partial(value) => {
                VarInt(0).encode(writer)?;
                value.encode(writer)?;
            }
            ResolvableProfileData::Complete(value) => {
                VarInt(1).encode(writer)?;
                value.encode(writer)?;
            }
        }
        self.body.encode(writer)?;
        self.cape.encode(writer)?;
        self.elytra.encode(writer)?;
        self.model.encode(writer)
    }
    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let profile = match VarInt::decode(reader)?.0 {
            0 => ResolvableProfileData::Partial(PartialProfile::decode(reader)?),
            1 => ResolvableProfileData::Complete(GameProfile::decode(reader)?),
            value => {
                return Err(CodecError::invalid_encoding(
                    CodecKind::StructuredComponent,
                    0,
                    InvalidEncodingReason::InvalidEnumValue {
                        value: i128::from(value),
                    },
                ));
            }
        };
        Ok(Self {
            profile,
            body: PrefixedOptional::decode(reader)?,
            cape: PrefixedOptional::decode(reader)?,
            elytra: PrefixedOptional::decode(reader)?,
            model: PrefixedOptional::decode(reader)?,
        })
    }
}

protocol_struct!(/// Inline painting variant data.
PaintingVariant {
    width: Int,
    height: Int,
    asset_id: Identifier,
    title: PrefixedOptional<TextComponent>,
    author: PrefixedOptional<TextComponent>,
});

/// A property predicate represented without invalid combinations of optional fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: PrefixedString,
    pub matcher: PropertyMatcher,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyMatcher {
    Exact(PrefixedString),
    Range {
        min: PrefixedString,
        max: PrefixedString,
    },
}

impl TypeCodec for Property {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.name.encode(writer)?;
        match &self.matcher {
            PropertyMatcher::Exact(value) => {
                Boolean(true).encode(writer)?;
                value.encode(writer)
            }
            PropertyMatcher::Range { min, max } => {
                Boolean(false).encode(writer)?;
                min.encode(writer)?;
                max.encode(writer)
            }
        }
    }
    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let name = PrefixedString::decode(reader)?;
        let matcher = if Boolean::decode(reader)?.0 {
            PropertyMatcher::Exact(PrefixedString::decode(reader)?)
        } else {
            PropertyMatcher::Range {
                min: PrefixedString::decode(reader)?,
                max: PrefixedString::decode(reader)?,
            }
        };
        Ok(Self { name, matcher })
    }
}

protocol_struct!(/// An exact typed data-component matcher.
    ExactDataComponentMatcher { component: DataComponent });

#[derive(Debug, Clone, Copy, PartialEq, Eq, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum PartialMatcherType {
    Damage = 0,
    Enchantments = 1,
    StoredEnchantments = 2,
    PotionContents = 3,
    CustomData = 4,
    Container = 5,
    BundleContents = 6,
    FireworkExplosion = 7,
    Fireworks = 8,
    WritableBookContent = 9,
    WrittenBookContent = 10,
    AttributeModifiers = 11,
    Trim = 12,
    JukeboxPlayable = 13,
}
protocol_struct!(/// An NBT-backed partial component predicate.
    PartialDataComponentMatcher { matcher_type: PartialMatcherType, predicate: Nbt });
protocol_struct!(/// A block predicate used by adventure-mode components.
BlockPredicate {
    blocks: PrefixedOptional<IdSet>,
    properties: PrefixedOptional<PrefixedArray<Property>>,
    nbt: PrefixedOptional<Nbt>,
    exact_components: PrefixedArray<ExactDataComponentMatcher>,
    partial_components: PrefixedArray<PartialDataComponentMatcher>,
});

protocol_struct!(/// Data for the apply-effects consume action.
    ApplyEffects { effects: PrefixedArray<PotionEffect>, probability: Float });
protocol_struct!(/// Data for the remove-effects consume action.
    RemoveEffects { effects: IdSet });
protocol_struct!(/// Data for random teleport consumption.
    TeleportRandomly { diameter: Float });
protocol_struct!(/// Data for play-sound consumption.
    PlaySound { sound: SoundEvent });

/// A type-safe consume-effect tagged union.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsumeEffect {
    ApplyEffects(ApplyEffects),
    RemoveEffects(RemoveEffects),
    ClearAllEffects,
    TeleportRandomly(TeleportRandomly),
    PlaySound(PlaySound),
}

impl TypeCodec for ConsumeEffect {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        match self {
            Self::ApplyEffects(v) => {
                VarInt(0).encode(writer)?;
                v.encode(writer)
            }
            Self::RemoveEffects(v) => {
                VarInt(1).encode(writer)?;
                v.encode(writer)
            }
            Self::ClearAllEffects => VarInt(2).encode(writer),
            Self::TeleportRandomly(v) => {
                VarInt(3).encode(writer)?;
                v.encode(writer)
            }
            Self::PlaySound(v) => {
                VarInt(4).encode(writer)?;
                v.encode(writer)
            }
        }
    }
    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        match VarInt::decode(reader)?.0 {
            0 => Ok(Self::ApplyEffects(ApplyEffects::decode(reader)?)),
            1 => Ok(Self::RemoveEffects(RemoveEffects::decode(reader)?)),
            2 => Ok(Self::ClearAllEffects),
            3 => Ok(Self::TeleportRandomly(TeleportRandomly::decode(reader)?)),
            4 => Ok(Self::PlaySound(PlaySound::decode(reader)?)),
            value => Err(CodecError::invalid_encoding(
                CodecKind::StructuredComponent,
                0,
                InvalidEncodingReason::InvalidEnumValue {
                    value: i128::from(value),
                },
            )),
        }
    }
}
