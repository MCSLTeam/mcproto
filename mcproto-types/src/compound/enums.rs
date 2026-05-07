use num_enum::{FromPrimitive, IntoPrimitive};
use crate::basic::VarInt;
use crate::{Codec, TypeCodecError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
#[repr(i32)]
pub enum Rarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
    #[num_enum(catch_all)]
    Unknown(i32) = -1,
}

impl Codec for Rarity {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        let id: i32 = (*self).into();
        VarInt(id).encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let id = VarInt::decode(buf)?.0;
        Self::try_from(id).map_err(|_| TypeCodecError::UnknownEnumValue(id, "Rarity".to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
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

impl Codec for PredicateType {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        VarInt(i32::from(*self)).encode(buf)?;
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let id = VarInt::decode(buf)?.0;
        Self::try_from(id).map_err(|_| TypeCodecError::UnknownEnumValue(id, "PredicateType".to_string()))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
#[repr(i32)]
pub enum AttributeOperation {
    Add = 0,
    MultiplyBase = 1,
    MultiplyTotal = 2,
    #[num_enum(catch_all)]
    Unknown(i32),
}
impl Codec for AttributeOperation {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        VarInt(i32::from(*self)).encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let id = VarInt::decode(buf)?.0;
        Self::try_from(id).map_err(|_| TypeCodecError::UnknownEnumValue(id, "AttributeOperation".to_string()))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, FromPrimitive)]
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

impl Codec for EquipmentSlot {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        VarInt(i32::from(*self)).encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let id = VarInt::decode(buf)?.0;
        Self::try_from(id).map_err(|_| TypeCodecError::UnknownEnumValue(id, "EquipmentSlot".to_string()))
    }
}