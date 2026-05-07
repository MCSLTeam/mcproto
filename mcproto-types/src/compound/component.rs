use crate::basic::Identifier;
use crate::compound::{Nbt, TextComponent, VarInt};
use crate::contextual::{PrefixedArray};
use crate::Codec;
use crate::TypeCodecError;
use num_enum::{FromPrimitive, IntoPrimitive};
use thiserror::__private18::Var;
use crate::compound::enums::ConsumeEffectData;
use crate::compound::subtypes::{AttributeModifier, BlockPredicate, Consumable, CustomModelData, Food, TooltipDisplay};
use crate::TypeCodecError::UnknownComponentType;

impl Codec for (VarInt, VarInt) {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.0.encode(buf)?;
        self.1.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok((VarInt::decode(buf)?, VarInt::decode(buf)?))
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
#[repr(i32)]
pub enum ComponentType {
    CustomData = 0,
    MaxStackSize = 1,
    MaxDamage = 2,
    Damage = 3,
    Unbreakable = 4,
    CustomName = 5,
    ItemName = 6,
    ItemModel = 7,
    Lore = 8,
    Rarity = 9,
    Enchantments = 10,
    CanPlaceOn = 11,
    CanBreak = 12,
    AttributeModifiers = 13,
    CustomModelData = 14,
    TooltipDisplay = 15,
    RepairCost = 16,
    CreativeSlotLock = 17,
    EnchantmentGlintOverride = 18,
    IntangibleProjectile = 19,
    Food = 20,
    Consumable = 21,
    UseRemainder = 22,
    UseCooldown = 23,
    DamageResistant = 24,
    Tool = 25,
    Weapon = 26,
    Enchantable = 27,
    Equippable = 28,
    Repairable = 29,
    Glider = 30,
    TooltipStyle = 31,
    DeathProtection = 32,
    BlocksAttacks = 33,
    StoredEnchantments = 34,
    DyedColor = 35,
    MapColor = 36,
    MapId = 37,
    MapDecorations = 38,
    MapPostProcessing = 39,
    ChargedProjectiles = 40,
    BundleContents = 41,
    PotionContents = 42,
    PotionDurationScale = 43,
    SuspiciousStewEffects = 44,
    WritableBookContent = 45,
    WrittenBookContent = 46,
    Trim = 47,
    DebugStickState = 48,
    EntityData = 49,
    BucketEntityData = 50,
    BlockEntityData = 51,
    Instrument = 52,
    ProvidesTrimMaterial = 53,
    OminousBottleAmplifier = 54,
    JukeboxPlayable = 55,
    ProvidesBannerPatterns = 56,
    Recipes = 57,
    LodestoneTracker = 58,
    FireworkExplosion = 59,
    Fireworks = 60,
    Profile = 61,
    NoteBlockSound = 62,
    BannerPatterns = 63,
    BaseColor = 64,
    PotDecorations = 65,
    Container = 66,
    BlockState = 67,
    Bees = 68,
    Lock = 69,
    ContainerLoot = 70,
    BreakSound = 71,
    VillagerVariant = 72,
    WolfVariant = 73,
    WolfSoundVariant = 74,
    WolfCollar = 75,
    FoxVariant = 76,
    SalmonSize = 77,
    ParrotVariant = 78,
    TropicalFishPattern = 79,
    TropicalFishBaseColor = 80,
    TropicalFishPatternColor = 81,
    MooshroomVariant = 82,
    RabbitVariant = 83,
    PigVariant = 84,
    CowVariant = 85,
    ChickenVariant = 86,
    FrogVariant = 87,
    HorseVariant = 88,
    PaintingVariant = 89,
    LlamaVariant = 90,
    AxolotlVariant = 91,
    CatVariant = 92,
    CatCollar = 93,
    SheepColor = 94,
    ShulkerColor = 95,
    #[num_enum(catch_all)]
    Unknown(i32) = -1,
}

impl Codec for ComponentType {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        let id: i32 = (*self).into();
        VarInt(id).encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let id = VarInt::decode(buf)?.0;
        Self::try_from(id).map_err(|_| TypeCodecError::UnknownComponentType(id as u8))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    CustomData(Nbt),
    MaxStackSize(VarInt),
    MaxDamage(VarInt),
    Damage(VarInt),
    Unbreakable,
    CustomName(TextComponent),
    ItemName(TextComponent),
    ItemModel(Identifier),
    Lore(PrefixedArray<TextComponent>),
    Rarity(VarInt),
    Enchantments(PrefixedArray<(VarInt, VarInt)>),
    // todo: 所有附魔的枚举
    CanPlaceOn(PrefixedArray<BlockPredicate>),
    CanBreak(PrefixedArray<BlockPredicate>),
    AttributeModifiers(AttributeModifier),
    CustomModelData(CustomModelData),
    TooltipDisplay(TooltipDisplay),
    RepairCost(VarInt),
    CreativeSlotLock,
    EnchantmentGlintOverride(bool),
    IntangibleProjectile(Nbt),
    Food(Food),
    Consumable(Consumable),
}
fn encode_component<T: Codec>(ty: ComponentType, value: &T, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
    ty.encode(buf)?;
    value.encode(buf)
}
impl Codec for Component {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        match self {
            Self::CustomData(v) => encode_component(ComponentType::CustomData, v, buf),
            Self::MaxStackSize(v) => encode_component(ComponentType::MaxStackSize, v, buf),
            Self::MaxDamage(v) => encode_component(ComponentType::MaxDamage, v, buf),
            Self::Damage(v) => encode_component(ComponentType::Damage, v, buf),
            Self::Unbreakable => ComponentType::Unbreakable.encode(buf),
            Self::CustomName(v) => encode_component(ComponentType::CustomName, v, buf),
            Self::ItemName(v) => encode_component(ComponentType::ItemName, v, buf),
            Self::ItemModel(v) => encode_component(ComponentType::ItemModel, v, buf),
            Self::Lore(v) => encode_component(ComponentType::Lore, v, buf),
            Self::Rarity(v) => encode_component(ComponentType::Rarity, v, buf),
            Self::Enchantments(v) => encode_component(ComponentType::Enchantments, v, buf),
            Self::CanPlaceOn(v) => encode_component(ComponentType::CanPlaceOn, v, buf),
            Self::CanBreak(v) => encode_component(ComponentType::CanBreak, v, buf),
            Self::AttributeModifiers(v) => encode_component(ComponentType::AttributeModifiers, v, buf),
            Self::CustomModelData(v) => encode_component(ComponentType::CustomModelData, v, buf),
            Self::TooltipDisplay(v) => encode_component(ComponentType::TooltipDisplay, v, buf),
            Self::RepairCost(v) => encode_component(ComponentType::RepairCost, v, buf),
            Self::CreativeSlotLock => ComponentType::CreativeSlotLock.encode(buf),
            Self::EnchantmentGlintOverride(v) => encode_component(ComponentType::EnchantmentGlintOverride, v, buf),
            Self::IntangibleProjectile(v) => encode_component(ComponentType::IntangibleProjectile, v, buf),
            Self::Food(v) => encode_component(ComponentType::Food, v, buf),
            Self::Consumable(v) => encode_component(ComponentType::Consumable, v, buf),
            _ => Err(UnknownComponentType(127)),
        }
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let comp_type = ComponentType::decode(buf)?;
        match comp_type {
            ComponentType::Unknown(id) => Err(UnknownComponentType(id as u8)),
            ComponentType::CustomData => Ok(Self::CustomData(Nbt::decode(buf)?)),
            ComponentType::MaxStackSize => Ok(Self::MaxStackSize(VarInt::decode(buf)?)),
            ComponentType::MaxDamage => Ok(Self::MaxDamage(VarInt::decode(buf)?)),
            ComponentType::Damage => Ok(Self::Damage(VarInt::decode(buf)?)),
            ComponentType::Unbreakable => Ok(Self::Unbreakable),
            ComponentType::CustomName => Ok(Self::CustomName(TextComponent::decode(buf)?)),
            ComponentType::ItemName => Ok(Self::ItemName(TextComponent::decode(buf)?)),
            ComponentType::ItemModel => Ok(Self::ItemModel(Identifier::decode(buf)?)),
            ComponentType::Lore => Ok(Self::Lore(PrefixedArray::decode(buf)?)),
            ComponentType::Rarity => Ok(Self::Rarity(VarInt::decode(buf)?)),
            ComponentType::Enchantments => Ok(Self::Enchantments(PrefixedArray::decode(buf)?)),
            ComponentType::CanPlaceOn => Ok(Self::CanPlaceOn(PrefixedArray::decode(buf)?)),
            ComponentType::CanBreak => Ok(Self::CanBreak(PrefixedArray::decode(buf)?)),
            ComponentType::AttributeModifiers => Ok(Self::AttributeModifiers(AttributeModifier::decode(buf)?)),
            ComponentType::CustomModelData => Ok(Self::CustomModelData(CustomModelData::decode(buf)?)),
            ComponentType::TooltipDisplay => Ok(Self::TooltipDisplay(TooltipDisplay::decode(buf)?)),
            ComponentType::RepairCost => Ok(Self::RepairCost(VarInt::decode(buf)?)),
            ComponentType::CreativeSlotLock => Ok(Self::CreativeSlotLock),
            ComponentType::EnchantmentGlintOverride => Ok(Self::EnchantmentGlintOverride(bool::decode(buf)?)),
            ComponentType::IntangibleProjectile => Ok(Self::IntangibleProjectile(Nbt::decode(buf)?)),
            ComponentType::Food => Ok(Self::Food(Food::decode(buf)?)),
            ComponentType::Consumable => Ok(Self::Consumable(Consumable::decode(buf)?)),


        }
    }
}
