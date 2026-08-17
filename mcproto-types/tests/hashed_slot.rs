//! Protocol tests for the Hashed Slot format.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    DataComponentType, HashedDataComponent, HashedItemStack, HashedSlot, Int, TypeCodec, VarInt,
};

#[test]
fn empty_hashed_slot_is_only_a_false_boolean() {
    let mut encoded = Vec::new();
    HashedSlot::Empty.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = [0x00, 0xaa].as_slice();
    assert_eq!(HashedSlot::decode(&mut input).unwrap(), HashedSlot::Empty);
    assert_eq!(input, [0xaa]);
}

#[test]
fn hashed_item_stack_matches_the_documented_wire_order() {
    let mut item = HashedItemStack::new(2, 5).unwrap();
    item.components_to_add = vec![
        HashedDataComponent {
            component_type: DataComponentType::MaxDamage,
            data_hash: Int(0x1234_5678),
        },
        HashedDataComponent {
            component_type: DataComponentType::Unbreakable,
            data_hash: Int(-1),
        },
    ];
    item.components_to_remove = vec![DataComponentType::Damage];
    let slot = HashedSlot::Item(item);

    let mut encoded = Vec::new();
    slot.encode(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        [
            0x01, // Has Item
            0x05, // Item ID
            0x02, // Item Count
            0x02, // Components to add
            0x02, 0x12, 0x34, 0x56, 0x78, // Max Damage and hash
            0x04, 0xff, 0xff, 0xff, 0xff, // Unbreakable and hash
            0x01, // Components to remove
            0x03, // Damage
        ]
    );

    let mut input = encoded.as_slice();
    assert_eq!(HashedSlot::decode(&mut input).unwrap(), slot);
    assert!(input.is_empty());
}

#[test]
fn invalid_presence_boolean_keeps_hashed_slot_context() {
    let error = HashedSlot::decode(&mut [0x02].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidBooleanValue { value: 2 })
    );
    assert_eq!(error.codec(), CodecKind::Boolean);
    assert_eq!(error.contexts(), &[CodecKind::HashedSlot]);
}

#[test]
fn non_positive_hashed_item_count_is_rejected() {
    let mut encoded = vec![0x01, 0x05];
    VarInt(-1).encode(&mut encoded).unwrap();

    let error = HashedSlot::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidSlotCount { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::HashedSlot);
}

#[test]
fn unknown_hashed_component_type_is_rejected() {
    let encoded = [
        0x01, // Has Item
        0x05, // Item ID
        0x01, // Item Count
        0x01, // Components to add
        0x6f, // Unknown component type 111
    ];

    let error = HashedSlot::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 111 })
    );
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(error.contexts(), &[CodecKind::HashedSlot]);
}
