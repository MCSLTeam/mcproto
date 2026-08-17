//! Protocol tests for Slot and structured data components.

use std::collections::HashSet;

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    Boolean, CustomModelDataComponent, DataComponent, DataComponentType, Float, Int, ItemStack,
    LockComponent, MaxDamageComponent, NbtString, Slot, TypeCodec, UnbreakableComponent,
    UseRemainderComponent, VarInt,
};

#[test]
fn empty_slot_is_only_a_zero_count() {
    let mut encoded = Vec::new();
    Slot::Empty.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x00]);

    let mut input = [0x00, 0xaa].as_slice();
    assert_eq!(Slot::decode(&mut input).unwrap(), Slot::Empty);
    assert_eq!(input, [0xaa]);
}

#[test]
fn item_stack_counts_are_derived_from_typed_vectors() {
    let mut item = ItemStack::new(2, 5).unwrap();
    item.components_to_add = vec![
        DataComponent::MaxDamage(MaxDamageComponent {
            max_damage: VarInt(100),
        }),
        DataComponent::Unbreakable(UnbreakableComponent),
    ];
    item.components_to_remove = vec![DataComponentType::Damage];
    let slot = Slot::Item(item);

    let mut encoded = Vec::new();
    slot.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x02, 0x05, 0x02, 0x01, 0x02, 0x64, 0x04, 0x03]);

    let mut input = encoded.as_slice();
    assert_eq!(Slot::decode(&mut input).unwrap(), slot);
    assert!(input.is_empty());
}

#[test]
fn recursive_slot_components_roundtrip() {
    let mut item = ItemStack::new(1, 2).unwrap();
    item.components_to_add = vec![DataComponent::UseRemainder(UseRemainderComponent {
        remainder: Box::new(Slot::Empty),
    })];
    let slot = Slot::Item(item);

    let mut encoded = Vec::new();
    slot.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x01, 0x02, 0x01, 0x00, 0x16, 0x00]);
    assert_eq!(Slot::decode(&mut encoded.as_slice()).unwrap(), slot);
}

#[test]
fn all_documented_component_type_ids_are_registered() {
    let mut names = HashSet::new();
    for id in 0_u8..=110 {
        let ty = DataComponentType::decode(&mut [id].as_slice()).unwrap();
        assert!(names.insert(ty.identifier()));
        let mut encoded = Vec::new();
        ty.encode(&mut encoded).unwrap();
        assert_eq!(encoded, [id]);
    }
    assert_eq!(names.len(), 111);
    assert_eq!(
        DataComponentType::CustomData.identifier(),
        "minecraft:custom_data"
    );
    assert_eq!(
        DataComponentType::ShulkerColor.identifier(),
        "minecraft:shulker/color"
    );
}

#[test]
fn unknown_component_type_is_rejected_before_payload_decode() {
    let error = DataComponentType::decode(&mut [111].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 111 })
    );
    assert_eq!(error.codec(), CodecKind::Enum);
}

#[test]
fn structured_component_fields_keep_wire_order() {
    let component = DataComponent::CustomModelData(CustomModelDataComponent {
        floats: vec![Float(1.0)].into(),
        flags: vec![Boolean(true), Boolean(false)].into(),
        strings: Vec::new().into(),
        colors: vec![Int(0x12_34_56)].into(),
    });
    let mut encoded = Vec::new();
    component.encode(&mut encoded).unwrap();

    assert_eq!(encoded[0], 14);
    let mut input = encoded.as_slice();
    assert_eq!(DataComponent::decode(&mut input).unwrap(), component);
    assert!(input.is_empty());
}

#[test]
fn negative_item_count_is_rejected() {
    let mut encoded = Vec::new();
    VarInt(-1).encode(&mut encoded).unwrap();
    let error = Slot::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidSlotCount { value: -1 })
    );
    assert_eq!(error.codec(), CodecKind::Slot);
}

#[test]
fn lock_component_uses_an_nbt_string_root() {
    let component = DataComponent::Lock(LockComponent {
        key: NbtString("key\0value".to_owned()),
    });
    let mut encoded = Vec::new();
    component.encode(&mut encoded).unwrap();

    assert_eq!(encoded[0], 78);
    assert_eq!(encoded[1], 8);
    assert!(encoded.windows(2).any(|bytes| bytes == [0xc0, 0x80]));
    assert_eq!(
        DataComponent::decode(&mut encoded.as_slice()).unwrap(),
        component
    );
}
