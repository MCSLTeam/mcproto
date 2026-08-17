//! Inventory slots, structured data components, and recipe slot displays.
//!
//! The implementation follows the current Java Edition protocol [Slot Data]
//! and [Slot Display] structures. Submodules live in `slot/`; this file is the
//! module root so no `slot/mod.rs` is used.
//!
//! [Slot Data]: https://minecraft.wiki/w/Java_Edition_protocol/Slot_data
//! [Slot Display]: https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    varint::{VarIntRead, VarIntWrite},
};

use crate::TypeCodec;

#[path = "slot/components.rs"]
pub mod components;
#[path = "slot/display.rs"]
pub mod display;
#[path = "slot/hashed.rs"]
pub mod hashed;
#[path = "slot/types.rs"]
pub mod types;

pub use components::*;
pub use display::*;
pub use hashed::*;
pub use types::*;

// Keep the established `slot::*` paths while using the canonical profile
// implementation owned by the profile module.
pub use crate::profile::{PartialProfile, ResolvableProfile, ResolvableProfileData, SkinModel};

/// A non-empty item stack carried by a [`Slot`].
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    /// Positive item count.
    pub count: u32,
    /// Numeric ID in the `minecraft:item` registry.
    pub item_id: u32,
    /// Typed component values added to or replacing item defaults.
    pub components_to_add: Vec<DataComponent>,
    /// Component types removed from item defaults.
    pub components_to_remove: Vec<DataComponentType>,
}

impl ItemStack {
    /// Creates an item stack, rejecting values that cannot be represented by a
    /// positive protocol VarInt.
    pub fn new(count: u32, item_id: u32) -> Result<Self, InvalidItemStack> {
        if count == 0 || count > i32::MAX as u32 {
            return Err(InvalidItemStack::Count(count));
        }
        if item_id > i32::MAX as u32 {
            return Err(InvalidItemStack::ItemId(item_id));
        }
        Ok(Self {
            count,
            item_id,
            components_to_add: Vec::new(),
            components_to_remove: Vec::new(),
        })
    }
}

/// An empty inventory slot or a complete non-empty item stack.
///
/// The item count is encoded first. Zero denotes [`Slot::Empty`]; a positive
/// count is followed by the item registry ID, component-add count,
/// component-remove count, typed added components, and removed component IDs.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     DataComponent, MaxDamageComponent, Slot, ItemStack, TypeCodec, VarInt,
/// };
///
/// let mut item = ItemStack::new(2, 5)?;
/// item.components_to_add.push(DataComponent::MaxDamage(
///     MaxDamageComponent { max_damage: VarInt(100) },
/// ));
/// let slot = Slot::Item(item);
/// let mut encoded = Vec::new();
/// slot.encode(&mut encoded)?;
///
/// let mut input = encoded.as_slice();
/// assert_eq!(Slot::decode(&mut input)?, slot);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Slot {
    /// Encoded as an item count of zero with no following fields.
    #[default]
    Empty,
    /// A positive item count followed by item and component-patch data.
    Item(ItemStack),
}

impl TypeCodec for Slot {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let Self::Item(item) = self else {
            return writer
                .write_varint(0)
                .map_err(|error| error.with_context(CodecKind::Slot));
        };
        validate_item(item)?;
        writer
            .write_varint(item.count as i32)
            .map_err(|error| error.with_context(CodecKind::Slot))?;
        writer
            .write_varint(item.item_id as i32)
            .map_err(|error| error.with_context(CodecKind::Slot))?;
        write_length(writer, item.components_to_add.len(), CodecKind::Slot)?;
        write_length(writer, item.components_to_remove.len(), CodecKind::Slot)?;
        for component in &item.components_to_add {
            component
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::Slot))?;
        }
        for component_type in &item.components_to_remove {
            component_type
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::Slot))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let count = reader
            .read_varint()
            .map_err(|error| error.with_context(CodecKind::Slot))?;
        if count == 0 {
            return Ok(Self::Empty);
        }
        let count = decode_item_count(count, CodecKind::Slot)?;
        let item_id = reader
            .read_varint()
            .map_err(|error| error.with_context(CodecKind::Slot))?;
        let item_id = decode_item_id(item_id, CodecKind::Slot)?;
        let add_count = read_length(reader, CodecKind::Slot)?;
        let remove_count = read_length(reader, CodecKind::Slot)?;
        let mut components_to_add = Vec::with_capacity(add_count.min(1024));
        for _ in 0..add_count {
            components_to_add.push(
                DataComponent::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::Slot))?,
            );
        }
        let mut components_to_remove = Vec::with_capacity(remove_count.min(1024));
        for _ in 0..remove_count {
            components_to_remove.push(
                DataComponentType::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::Slot))?,
            );
        }
        Ok(Self::Item(ItemStack {
            count,
            item_id,
            components_to_add,
            components_to_remove,
        }))
    }
}

fn validate_item(item: &ItemStack) -> Result<(), CodecError> {
    validate_item_fields(item.count, item.item_id, CodecKind::Slot)
}

pub(super) fn validate_item_fields(
    count: u32,
    item_id: u32,
    kind: CodecKind,
) -> Result<(), CodecError> {
    if count == 0 || count > i32::MAX as u32 {
        return Err(CodecError::invalid_encoding_for_operation(
            kind,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::InvalidSlotCount {
                value: i64::from(count),
            },
        ));
    }
    if item_id > i32::MAX as u32 {
        return Err(CodecError::invalid_encoding_for_operation(
            kind,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::InvalidRegistryId {
                value: item_id as i32,
                max: i32::MAX,
            },
        ));
    }
    Ok(())
}

pub(super) fn write_length(
    writer: &mut impl Write,
    length: usize,
    kind: CodecKind,
) -> Result<(), CodecError> {
    let length = i32::try_from(length).map_err(|_| {
        CodecError::invalid_encoding_for_operation(
            kind,
            CodecOperation::Write,
            0,
            InvalidEncodingReason::LengthOutOfRange {
                max: i32::MAX as usize,
                actual: length,
            },
        )
    })?;
    writer
        .write_varint(length)
        .map_err(|error| error.with_context(kind))
}

pub(super) fn read_length(reader: &mut impl Read, kind: CodecKind) -> Result<usize, CodecError> {
    let value = reader
        .read_varint()
        .map_err(|error| error.with_context(kind))?;
    usize::try_from(value).map_err(|_| {
        CodecError::invalid_encoding(kind, 0, InvalidEncodingReason::NegativeLength { value })
    })
}

pub(super) fn decode_item_count(value: i32, kind: CodecKind) -> Result<u32, CodecError> {
    if value <= 0 {
        return Err(CodecError::invalid_encoding(
            kind,
            0,
            InvalidEncodingReason::InvalidSlotCount {
                value: i64::from(value),
            },
        ));
    }
    Ok(value as u32)
}

pub(super) fn decode_item_id(value: i32, kind: CodecKind) -> Result<u32, CodecError> {
    if value < 0 {
        return Err(CodecError::invalid_encoding(
            kind,
            0,
            InvalidEncodingReason::InvalidRegistryId {
                value,
                max: i32::MAX,
            },
        ));
    }
    Ok(value as u32)
}

/// Error returned when constructing an invalid non-empty item stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidItemStack {
    /// The item count is zero or exceeds a positive VarInt.
    Count(u32),
    /// The item registry ID exceeds a non-negative VarInt.
    ItemId(u32),
}

impl std::fmt::Display for InvalidItemStack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(value) => write!(formatter, "invalid item count: {value}"),
            Self::ItemId(value) => write!(formatter, "invalid item registry ID: {value}"),
        }
    }
}

impl std::error::Error for InvalidItemStack {}
