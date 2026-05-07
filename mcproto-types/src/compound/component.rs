use crate::basic::Identifier;
use crate::compound::{Nbt, TextComponent, VarInt};
use crate::contextual::{PrefixedArray};
use crate::Codec;
use crate::TypeCodecError;
use num_enum::{FromPrimitive, IntoPrimitive};
use crate::compound::subtypes::{AttributeModifier, BlockPredicate};

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
            Self::AttributeModifiers(v) => encode_component(ComponentType::AttributeModifiers, v, buf)
        }
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let comp_type = ComponentType::decode(buf)?;
        match comp_type {
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
            ComponentType::Unknown(id) => Err(TypeCodecError::UnknownComponentType(id as u8)),

        }
    }
}
