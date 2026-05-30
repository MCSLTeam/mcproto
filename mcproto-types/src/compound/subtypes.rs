use crate::basic::{Double, Float, Identifier, Int, VarInt};
use crate::compound::structured_component::{Component, ComponentType};
use crate::compound::enums::{AttributeOperation, ConsumableAnimation, ConsumeEffectData, EquipmentSlot, PredicateType};
use crate::compound::Nbt;
use crate::contextual::{IdOr, IdSet, Optional, PrefixedArray, PrefixedOptional, SoundEvent};
use crate::{Codec, ContextualCodec, Ctx, TypeCodecError};
use mcproto_derive::Codec;

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct BlockPredicate {
    pub blocks: PrefixedOptional<IdSet>,
    pub properties: PrefixedOptional<PrefixedArray<Property>>,
    pub nbt: PrefixedOptional<Nbt>,
    pub data_components: PrefixedArray<ExactDataComponentMatcher>,
    pub partial_data_component_predicates: PrefixedArray<PartialDataComponentMatcher>,
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

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct ExactDataComponentMatcher {
    pub component_type: ComponentType,
    pub value: Box<Component>,
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct PartialDataComponentMatcher {
    pub predicate_type: PredicateType,  // VarInt Enum
    pub predicate: Nbt,
}


#[derive(Debug, Clone, PartialEq, Codec)]
pub struct AttributeModifier {
    pub attribute_id: VarInt,
    pub modifier_id: Identifier,
    pub value: Double,
    pub operation: AttributeOperation,
    pub slot: EquipmentSlot,
}



#[derive(Debug, Clone, PartialEq, Codec)]
pub struct CustomModelData {
    pub floats: PrefixedArray<Float>,
    pub flags: PrefixedArray<bool>,
    pub strings: PrefixedArray<String>,
    pub colors: PrefixedArray<Int>,
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct TooltipDisplay {
    pub hide_tooltip: bool,
    pub hidden_components: PrefixedArray<ComponentType>
}


#[derive(Debug, Clone, PartialEq, Codec)]
pub struct Food {
    pub nutrition: VarInt,
    pub saturation_modifier: Float,
    pub can_always_eat: bool
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct PotionEffect {
    pub type_id: VarInt,
    pub details: PotionEffectDetail
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct PotionEffectDetail {
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub hidden_effect: Box<PotionEffectDetail>
}
#[derive(Debug, Clone, PartialEq, Codec)]
pub struct ConsumeEffect {
    pub r#type: VarInt,
    pub data: ConsumeEffectData
}
#[derive(Debug, Clone, PartialEq, Codec)]
pub struct Consumable {
    pub consume_seconds: Float,
    pub animation: ConsumableAnimation,
    pub sound: IdOr<SoundEvent>,
    pub has_consume_particles: bool,
    pub effects: PrefixedArray<ConsumeEffect>
}
#[derive(Debug, Clone, PartialEq, Codec)]
pub struct Cooldown {
    pub seconds: Float,
    pub cooldown_group: PrefixedOptional<Identifier>
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct ToolRule {
    pub blocks: IdSet,
    pub speed: PrefixedOptional<Float>,
    pub correct_drop_for_blocks: PrefixedOptional<bool>
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct Tool {
    pub rule: PrefixedArray<ToolRule>,
    pub default_mining_speed: Float,
    pub damage_per_block: VarInt,
    pub can_destroy_blocks_in_creative: bool,
}

#[derive(Debug, Clone, PartialEq, Codec)]
pub struct Weapon {
    pub damage_per_attack: VarInt,
    pub disabling_block_for: Float,
}
