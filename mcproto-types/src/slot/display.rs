//! Type-safe recipe slot display tagged union.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind, InvalidEncodingReason};

use crate::{Identifier, PrefixedArray, TypeCodec, TypeStructCodec, VarInt};

use super::{Slot, components::DataComponentType};

macro_rules! display_struct {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = SlotDisplay)]
        pub struct $name { $($(#[$field_meta])* pub $field: $ty,)* }
    };
}

display_struct!(/// Applies any potion to a base display.
    WithAnyPotionSlotDisplay { base: Box<SlotDisplay> });
display_struct!(/// Shows a base only with a specific component type.
OnlyWithComponentSlotDisplay {
    base: Box<SlotDisplay>,
    component_type: DataComponentType,
});
display_struct!(/// Displays an item registry entry.
    ItemSlotDisplay { item_type_id: VarInt });
display_struct!(/// Displays a complete item stack.
    ItemStackSlotDisplay { item_stack: Slot });
display_struct!(/// Displays the members of an item tag.
    TagSlotDisplay { tag: Identifier });
display_struct!(/// Displays dye and target slots.
    DyedSlotDisplay { dye: Box<SlotDisplay>, target: Box<SlotDisplay> });
display_struct!(/// Displays a smithing trim preview.
SmithingTrimSlotDisplay {
    base: Box<SlotDisplay>,
    material: Box<SlotDisplay>,
    pattern_id: VarInt,
});
display_struct!(/// Displays an ingredient together with its remainder.
WithRemainderSlotDisplay {
    ingredient: Box<SlotDisplay>,
    remainder: Box<SlotDisplay>,
});
display_struct!(/// Displays a choice among multiple slot displays.
    CompositeSlotDisplay { options: PrefixedArray<SlotDisplay> });

/// Description of a recipe ingredient slot for use by the client.
///
/// The enum variant determines both the registry type ID and the exact payload,
/// so mismatched type IDs and payloads cannot be represented in memory.
/// See the official [Slot Display structure] documentation.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     CompositeSlotDisplay, ItemSlotDisplay, PrefixedArray, SlotDisplay, TypeCodec, VarInt,
/// };
///
/// let display = SlotDisplay::Composite(CompositeSlotDisplay {
///     options: PrefixedArray(vec![
///         SlotDisplay::AnyFuel,
///         SlotDisplay::Item(ItemSlotDisplay { item_type_id: VarInt(5) }),
///     ]),
/// });
/// let mut encoded = Vec::new();
/// display.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(SlotDisplay::decode(&mut input)?, display);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// [Slot Display structure]: https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure
#[derive(Debug, Clone, PartialEq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion(WithAnyPotionSlotDisplay),
    OnlyWithComponent(OnlyWithComponentSlotDisplay),
    Item(ItemSlotDisplay),
    ItemStack(ItemStackSlotDisplay),
    Tag(TagSlotDisplay),
    Dyed(DyedSlotDisplay),
    SmithingTrim(SmithingTrimSlotDisplay),
    WithRemainder(WithRemainderSlotDisplay),
    Composite(CompositeSlotDisplay),
}

impl TypeCodec for SlotDisplay {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let (id, payload): (i32, Option<&dyn EncodableDisplay>) = match self {
            Self::Empty => (0, None),
            Self::AnyFuel => (1, None),
            Self::WithAnyPotion(v) => (2, Some(v)),
            Self::OnlyWithComponent(v) => (3, Some(v)),
            Self::Item(v) => (4, Some(v)),
            Self::ItemStack(v) => (5, Some(v)),
            Self::Tag(v) => (6, Some(v)),
            Self::Dyed(v) => (7, Some(v)),
            Self::SmithingTrim(v) => (8, Some(v)),
            Self::WithRemainder(v) => (9, Some(v)),
            Self::Composite(v) => (10, Some(v)),
        };
        VarInt(id)
            .encode(writer)
            .map_err(|e| e.with_context(CodecKind::SlotDisplay))?;
        if let Some(payload) = payload {
            payload
                .encode_display(writer)
                .map_err(|e| e.with_context(CodecKind::SlotDisplay))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let id = VarInt::decode(reader)
            .map_err(|e| e.with_context(CodecKind::SlotDisplay))?
            .0;
        match id {
            0 => Ok(Self::Empty),
            1 => Ok(Self::AnyFuel),
            2 => Ok(Self::WithAnyPotion(WithAnyPotionSlotDisplay::decode(
                reader,
            )?)),
            3 => Ok(Self::OnlyWithComponent(
                OnlyWithComponentSlotDisplay::decode(reader)?,
            )),
            4 => Ok(Self::Item(ItemSlotDisplay::decode(reader)?)),
            5 => Ok(Self::ItemStack(ItemStackSlotDisplay::decode(reader)?)),
            6 => Ok(Self::Tag(TagSlotDisplay::decode(reader)?)),
            7 => Ok(Self::Dyed(DyedSlotDisplay::decode(reader)?)),
            8 => Ok(Self::SmithingTrim(SmithingTrimSlotDisplay::decode(reader)?)),
            9 => Ok(Self::WithRemainder(WithRemainderSlotDisplay::decode(
                reader,
            )?)),
            10 => Ok(Self::Composite(CompositeSlotDisplay::decode(reader)?)),
            value => Err(CodecError::invalid_encoding(
                CodecKind::SlotDisplay,
                0,
                InvalidEncodingReason::InvalidEnumValue {
                    value: i128::from(value),
                },
            )),
        }
    }
}

trait EncodableDisplay {
    fn encode_display(&self, writer: &mut dyn Write) -> Result<(), CodecError>;
}

macro_rules! impl_encodable_display {
    ($($ty:ty),+ $(,)?) => {$(
        impl EncodableDisplay for $ty {
            fn encode_display(&self, writer: &mut dyn Write) -> Result<(), CodecError> {
                struct DynWriter<'a>(&'a mut dyn Write);
                impl Write for DynWriter<'_> {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.write(buf) }
                    fn flush(&mut self) -> std::io::Result<()> { self.0.flush() }
                }
                self.encode(&mut DynWriter(writer))
            }
        }
    )+};
}

impl_encodable_display!(
    WithAnyPotionSlotDisplay,
    OnlyWithComponentSlotDisplay,
    ItemSlotDisplay,
    ItemStackSlotDisplay,
    TagSlotDisplay,
    DyedSlotDisplay,
    SmithingTrimSlotDisplay,
    WithRemainderSlotDisplay,
    CompositeSlotDisplay,
);
