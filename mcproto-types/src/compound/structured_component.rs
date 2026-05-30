use crate::basic::Identifier;
use crate::compound::{Nbt, Slot, TextComponent, VarInt};
use crate::contextual::{PrefixedArray};
use crate::Codec;
use crate::TypeCodecError;
use num_enum::{FromPrimitive, IntoPrimitive};
use thiserror::__private18::Var;
use mcproto_derive::ComponentCodec;
use crate::compound::enums::ConsumeEffectData;
use crate::compound::subtypes::{AttributeModifier, BlockPredicate, Consumable, Cooldown, CustomModelData, Food, Tool, TooltipDisplay, Weapon};

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


#[derive(Debug, Clone, PartialEq, ComponentCodec)]
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
    UseRemainder(Slot),
    UseCooldown(Cooldown),
    DamageResistant(Identifier),
    Tool(Tool),
    Weapon(Weapon),
    Enchantable(VarInt),

}
