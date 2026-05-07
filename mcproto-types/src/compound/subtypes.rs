use crate::basic::{Double, Float, Identifier, Int, VarInt};
use crate::compound::component::{Component, ComponentType};
use crate::compound::enums::{AttributeOperation, ConsumableAnimation, ConsumeEffectData, EquipmentSlot, PredicateType};
use crate::compound::Nbt;
use crate::contextual::{IdOr, IdSet, Optional, PrefixedArray, PrefixedOptional, SoundEvent};
use crate::{Codec, ContextualCodec, Ctx, TypeCodecError};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPredicate {
    pub blocks: PrefixedOptional<IdSet>,
    pub properties: PrefixedOptional<PrefixedArray<Property>>,
    pub nbt: PrefixedOptional<Nbt>,
    pub data_components: PrefixedArray<ExactDataComponentMatcher>,
    pub partial_data_component_predicates: PrefixedArray<PartialDataComponentMatcher>,
}
impl Codec for BlockPredicate {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.blocks.encode(buf)?;
        self.properties.encode(buf)?;
        self.nbt.encode(buf)?;
        self.data_components.encode(buf)?;
        self.partial_data_component_predicates.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(BlockPredicate {
            blocks: PrefixedOptional::<IdSet>::decode(buf)?,
            properties: PrefixedOptional::<PrefixedArray<Property>>::decode(buf)?,
            nbt: PrefixedOptional::<Nbt>::decode(buf)?,
            data_components: PrefixedArray::<ExactDataComponentMatcher>::decode(buf)?,
            partial_data_component_predicates: PrefixedArray::<PartialDataComponentMatcher>::decode(buf)?,
        })
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub is_exact_match: bool,
    pub exact_value: Optional<String>,
    pub min_value: Optional<String>,
    pub max_value: Optional<String>,
}
// Property
impl Codec for Property {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.name.encode(buf)?;
        self.is_exact_match.encode(buf)?;
        if self.is_exact_match {
            self.exact_value.encode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?;
        } else {
            self.min_value.encode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?;
            self.max_value.encode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?;
        }
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let name = String::decode(buf)?;
        let is_exact_match = bool::decode(buf)?;
        if is_exact_match {
            Ok(Property {
                name,
                is_exact_match,
                exact_value: Optional::decode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?,
                min_value: Optional::new(None),
                max_value: Optional::new(None),
            })
        } else {
            Ok(Property {
                name,
                is_exact_match,
                exact_value: Optional::new(None),
                min_value: Optional::decode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?,
                max_value: Optional::decode_with_ctx(buf, &Ctx { present: Some(true), ..Default::default() })?,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExactDataComponentMatcher {
    pub component_type: ComponentType,
    pub value: Box<Component>,
}
// ExactDataComponentMatcher
impl Codec for ExactDataComponentMatcher {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.component_type.encode(buf)?;
        self.value.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(ExactDataComponentMatcher {
            component_type: ComponentType::decode(buf)?,
            value: Box::new(Component::decode(buf)?),
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct PartialDataComponentMatcher {
    pub predicate_type: PredicateType,  // VarInt Enum
    pub predicate: Nbt,
}
// PartialDataComponentMatcher
impl Codec for PartialDataComponentMatcher {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.predicate_type.encode(buf)?;
        self.predicate.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(PartialDataComponentMatcher {
            predicate_type: PredicateType::decode(buf)?,
            predicate: Nbt::decode(buf)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub attribute_id: VarInt,
    pub modifier_id: Identifier,
    pub value: Double,
    pub operation: AttributeOperation,
    pub slot: EquipmentSlot,
}


// AttributeModifier
impl Codec for AttributeModifier {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.attribute_id.encode(buf)?;
        self.modifier_id.encode(buf)?;
        self.value.encode(buf)?;
        self.operation.encode(buf)?;
        self.slot.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Ok(AttributeModifier {
            attribute_id: VarInt::decode(buf)?,
            modifier_id: Identifier::decode(buf)?,
            value: Double::decode(buf)?,
            operation: AttributeOperation::decode(buf)?,
            slot: EquipmentSlot::decode(buf)?,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct CustomModelData {
    pub floats: PrefixedArray<Float>,
    pub flags: PrefixedArray<bool>,
    pub strings: PrefixedArray<String>,
    pub colors: PrefixedArray<Int>,
}
impl Codec for CustomModelData {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.floats.encode(buf)?;
        self.flags.encode(buf)?;
        self.strings.encode(buf)?;
        self.colors.encode(buf)
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let floats = PrefixedArray::decode(buf)?;
        let flags = PrefixedArray::decode(buf)?;
        let strings = PrefixedArray::decode(buf)?;
        let colors = PrefixedArray::decode(buf)?;
        Ok(CustomModelData {
            floats,
            flags,
            strings,
            colors
        })

    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipDisplay {
    pub hide_tooltip: bool,
    pub hidden_components: PrefixedArray<ComponentType>
}

impl Codec for TooltipDisplay {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.hide_tooltip.encode(buf)?;
        self.hidden_components.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let hide_tooltip = bool::decode(buf)?;
        let hidden_components = PrefixedArray::decode(buf)?;
        Ok(TooltipDisplay {
            hide_tooltip,
            hidden_components
        })

    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Food {
    pub nutrition: VarInt,
    pub saturation_modifier: Float,
    pub can_always_eat: bool
}
impl Codec for Food {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.nutrition.encode(buf)?;
        self.saturation_modifier.encode(buf)?;
        self.can_always_eat.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let nutrition = VarInt::decode(buf)?;
        let saturation_modifier = Float::decode(buf)?;
        let can_always_eat = bool::decode(buf)?;
        Ok(Food {
            nutrition,
            saturation_modifier,
            can_always_eat
        })
    }

}
#[derive(Debug, Clone, PartialEq)]
pub struct PotionEffect {
    pub type_id: VarInt,
    pub details: PotionEffectDetail
}
impl Codec for PotionEffect {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.type_id.encode(buf)?;
        self.details.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let type_id = VarInt::decode(buf)?;
        let details = PotionEffectDetail::decode(buf)?;
        Ok(PotionEffect {
            type_id,
            details
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PotionEffectDetail {
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub hidden_effect: Box<PotionEffectDetail>
}
impl Codec for PotionEffectDetail {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.amplifier.encode(buf)?;
        self.duration.encode(buf)?;
        self.ambient.encode(buf)?;
        self.show_particles.encode(buf)?;
        self.show_icon.encode(buf)?;
        self.hidden_effect.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let amplifier = VarInt::decode(buf)?;
        let duration = VarInt::decode(buf)?;
        let ambient = bool::decode(buf)?;
        let show_particles = bool::decode(buf)?;
        let show_icon = bool::decode(buf)?;
        let hidden_effect = Box::new(PotionEffectDetail::decode(buf)?);
        Ok(PotionEffectDetail {
            amplifier,
            duration,
            ambient,
            show_particles,
            show_icon,
            hidden_effect
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumeEffect {
    pub r#type: VarInt,
    pub data: ConsumeEffectData
}
impl Codec for ConsumeEffect {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.r#type.encode(buf)?;
        self.data.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let r#type = VarInt::decode(buf)?;
        let data = ConsumeEffectData::decode(buf)?;
        Ok(ConsumeEffect {
            r#type,
            data
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Consumable {
    pub consume_seconds: Float,
    pub animation: ConsumableAnimation,
    pub sound: IdOr<SoundEvent>,
    pub has_consume_particles: bool,
    pub effects: PrefixedArray<ConsumeEffect>
}
impl Codec for Consumable {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.consume_seconds.encode(buf)?;
        self.animation.encode(buf)?;
        self.sound.encode(buf)?;
        self.has_consume_particles.encode(buf)?;
        self.effects.encode(buf)?;
        Ok(())
    }
    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let consume_seconds = Float::decode(buf)?;
        let animation = ConsumableAnimation::decode(buf)?;
        let sound = IdOr::<SoundEvent>::decode(buf)?;
        let has_consume_particles = bool::decode(buf)?;
        let effects = PrefixedArray::decode(buf)?;
        Ok(Consumable {
            consume_seconds,
            animation,
            sound,
            has_consume_particles,
            effects
        })
    }

}
