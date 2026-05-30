use crate::basic::{Float, Identifier, VarInt};
use crate::compound::subtypes::{PotionEffect, TrimMaterial};
use crate::contextual::{PrefixedArray, SoundEvent};
use crate::{Codec, TypeCodecError};
use num_enum::{FromPrimitive, IntoPrimitive};
use mcproto_codec::VarIntRead;
use mcproto_derive::VarIntEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum Rarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
    #[num_enum(catch_all)]
    Unknown(i32) = -1,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum PredicateType {
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
    #[num_enum(catch_all)]
    Unknown(i32) = -1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum AttributeOperation {
    Add = 0,
    MultiplyBase = 1,
    MultiplyTotal = 2,
    #[num_enum(catch_all)]
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum EquipmentSlot {
    Any = 0,
    Mainhand = 1,
    Offhand = 2,
    Hand = 3,
    Feet = 4,
    Legs = 5,
    Chest = 6,
    Head = 7,
    Armor = 8,
    Body = 9,
    #[num_enum(catch_all)]
    Unknown(i32),
}

// VarInt Enum 	0: none, 1: eat, 2: drink, 3: block, 4: bow, 5: spear, 6: crossbow, 7: spyglass, 8: toot_horn, 9: brush
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum ConsumableAnimation {
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
    #[num_enum(catch_all)]
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum MapPostProcessing {
    Lock = 0,
    Scale = 1,
    #[num_enum(catch_all)]
    Unknown(i32),
}


#[derive(Debug, Clone, PartialEq)]
pub enum ConsumeEffectData {
    ApplyEffects(PrefixedArray<PotionEffect>, Float),
    RemoveEffects(PrefixedArray<PotionEffect>),
    ClearAllEffects,
    TeleportRandomly(Float),
    PlaySound(SoundEvent)

}
impl Codec for ConsumeEffectData {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        match self {
            ConsumeEffectData::ApplyEffects(effect_array, effect) => {
                effect_array.encode(buf)?;
                effect.encode(buf)?;
                Ok(())
            }
            ConsumeEffectData::RemoveEffects(effect_array) => {
                effect_array.encode(buf)?;
                Ok(())
            }
            ConsumeEffectData::ClearAllEffects => {
                Ok(())
            }
            ConsumeEffectData::TeleportRandomly(effect) => {
                effect.encode(buf)?;
                Ok(())
            }
            ConsumeEffectData::PlaySound(effect) => {
                effect.encode(buf)?;
                Ok(())
            }
        }
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError>
    where
        Self: Sized,
    {
        let tag = buf.read_varint()?;
        match tag {
            0 => Ok(ConsumeEffectData::ApplyEffects(PrefixedArray::decode(buf)?, Float::decode(buf)?)),
            1 => Ok(ConsumeEffectData::RemoveEffects(PrefixedArray::decode(buf)?)),
            2 => Ok(ConsumeEffectData::ClearAllEffects),
            3 => Ok(ConsumeEffectData::TeleportRandomly(Float::decode(buf)?)),
            4 => Ok(ConsumeEffectData::PlaySound(SoundEvent::decode(buf)?)),
            _ => Err(TypeCodecError::UnknownEnumValue(tag, "ConsumeEffectType".to_string())),
            
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive, VarIntEnum)]
#[repr(i32)]
pub enum TrimMaterialMode {
    Reference = 0,  // 直接用 Identifier
    Inline = 1,     // 内嵌 TrimMaterial 定义
    #[num_enum(catch_all)]
    Unknown(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrimMaterialData {
    Reference(Identifier),
    Inline(TrimMaterial),
}
impl Codec for TrimMaterialData {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        match self {
            TrimMaterialData::Reference(id) => id.encode(buf),
            TrimMaterialData::Inline(mat) => mat.encode(buf),
        }
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Err(TypeCodecError::MissingContext("mode"))
    }
}

