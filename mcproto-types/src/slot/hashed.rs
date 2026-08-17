//! Hashed inventory slots used by serverbound container interactions.

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind},
    varint::{VarIntRead, VarIntWrite},
};

use crate::{Boolean, Int, TypeCodec, TypeStructCodec};

use super::{
    DataComponentType, InvalidItemStack, decode_item_count, decode_item_id, read_length,
    validate_item_fields, write_length,
};

/// A data component type and the CRC32C hash of its encoded value.
///
/// The protocol defines the hash as an [`Int`] bit pattern. How vanilla
/// computes this CRC32C value is currently undocumented; this type stores and
/// transmits an already-computed hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = HashedSlot)]
pub struct HashedDataComponent {
    /// Type of the component whose value was hashed.
    pub component_type: DataComponentType,
    /// CRC32C hash represented by the protocol's signed 32-bit `Int` field.
    pub data_hash: Int,
}

/// A non-empty item stack carried by a [`HashedSlot`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedItemStack {
    /// Numeric ID in the `minecraft:item` registry.
    pub item_id: u32,
    /// Positive item count.
    pub count: u32,
    /// Component types and CRC32C hashes added to the item defaults.
    pub components_to_add: Vec<HashedDataComponent>,
    /// Component types removed from the item defaults.
    pub components_to_remove: Vec<DataComponentType>,
}

impl HashedItemStack {
    /// Creates a hashed item stack with no component changes.
    ///
    /// The count must be positive and both fields must fit their non-negative
    /// VarInt wire representations.
    pub fn new(count: u32, item_id: u32) -> Result<Self, InvalidItemStack> {
        if count == 0 || count > i32::MAX as u32 {
            return Err(InvalidItemStack::Count(count));
        }
        if item_id > i32::MAX as u32 {
            return Err(InvalidItemStack::ItemId(item_id));
        }
        Ok(Self {
            item_id,
            count,
            components_to_add: Vec::new(),
            components_to_remove: Vec::new(),
        })
    }
}

/// An empty slot or an item stack represented using component data hashes.
///
/// A boolean is written first. `false` represents an empty slot. When it is
/// `true`, the item ID and count are followed by a prefixed array of component
/// type/hash pairs and a prefixed array of removed component types. Array order
/// is preserved exactly.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     DataComponentType, HashedDataComponent, HashedItemStack, HashedSlot, Int, TypeCodec,
/// };
///
/// let mut item = HashedItemStack::new(2, 5)?;
/// item.components_to_add.push(HashedDataComponent {
///     component_type: DataComponentType::MaxDamage,
///     data_hash: Int(0x1234_5678),
/// });
/// let slot = HashedSlot::Item(item);
///
/// let mut encoded = Vec::new();
/// slot.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(HashedSlot::decode(&mut input)?, slot);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// See the official [Hashed Format] protocol documentation.
///
/// [Hashed Format]: https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Hashed_Format
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HashedSlot {
    /// Encoded only as a `false` presence boolean.
    #[default]
    Empty,
    /// Encoded as `true` followed by the hashed item-stack fields.
    Item(HashedItemStack),
}

impl TypeCodec for HashedSlot {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let Self::Item(item) = self else {
            return Boolean(false)
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::HashedSlot));
        };

        validate_item_fields(item.count, item.item_id, CodecKind::HashedSlot)?;
        Boolean(true)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        writer
            .write_varint(item.item_id as i32)
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        writer
            .write_varint(item.count as i32)
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        write_length(writer, item.components_to_add.len(), CodecKind::HashedSlot)?;
        for component in &item.components_to_add {
            component.encode(writer)?;
        }
        write_length(
            writer,
            item.components_to_remove.len(),
            CodecKind::HashedSlot,
        )?;
        for component_type in &item.components_to_remove {
            component_type
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let has_item = Boolean::decode(reader)
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?
            .0;
        if !has_item {
            return Ok(Self::Empty);
        }

        let item_id = reader
            .read_varint()
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        let item_id = decode_item_id(item_id, CodecKind::HashedSlot)?;
        let count = reader
            .read_varint()
            .map_err(|error| error.with_context(CodecKind::HashedSlot))?;
        let count = decode_item_count(count, CodecKind::HashedSlot)?;

        let add_count = read_length(reader, CodecKind::HashedSlot)?;
        let mut components_to_add = Vec::with_capacity(add_count.min(1024));
        for _ in 0..add_count {
            components_to_add.push(HashedDataComponent::decode(reader)?);
        }

        let remove_count = read_length(reader, CodecKind::HashedSlot)?;
        let mut components_to_remove = Vec::with_capacity(remove_count.min(1024));
        for _ in 0..remove_count {
            components_to_remove.push(
                DataComponentType::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::HashedSlot))?,
            );
        }

        Ok(Self::Item(HashedItemStack {
            item_id,
            count,
            components_to_add,
            components_to_remove,
        }))
    }
}
