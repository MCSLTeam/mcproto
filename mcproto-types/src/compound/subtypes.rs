use crate::basic::{Double, Identifier, VarInt};
use crate::compound::component::{Component, ComponentType};
use crate::compound::enums::{AttributeOperation, EquipmentSlot, PredicateType};
use crate::compound::Nbt;
use crate::contextual::{IdSet, Optional, PrefixedArray, PrefixedOptional};
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
