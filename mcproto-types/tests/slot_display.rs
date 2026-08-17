//! Protocol tests for Slot Display.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    CompositeSlotDisplay, DataComponentType, ItemSlotDisplay, ItemStackSlotDisplay,
    OnlyWithComponentSlotDisplay, PrefixedArray, Slot, SlotDisplay, TypeCodec, VarInt,
};

#[test]
fn no_data_displays_are_single_varints() {
    for (value, expected) in [(SlotDisplay::Empty, 0), (SlotDisplay::AnyFuel, 1)] {
        let mut encoded = Vec::new();
        value.encode(&mut encoded).unwrap();
        assert_eq!(encoded, [expected]);
        assert_eq!(SlotDisplay::decode(&mut encoded.as_slice()).unwrap(), value);
    }
}

#[test]
fn item_and_item_stack_payloads_follow_the_type_id() {
    let item = SlotDisplay::Item(ItemSlotDisplay {
        item_type_id: VarInt(300),
    });
    let mut encoded = Vec::new();
    item.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x04, 0xac, 0x02]);

    let stack = SlotDisplay::ItemStack(ItemStackSlotDisplay {
        item_stack: Slot::Empty,
    });
    let mut encoded = Vec::new();
    stack.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x05, 0x00]);
}

#[test]
fn recursive_composite_roundtrips_and_preserves_order() {
    let value = SlotDisplay::Composite(CompositeSlotDisplay {
        options: PrefixedArray(vec![
            SlotDisplay::AnyFuel,
            SlotDisplay::Item(ItemSlotDisplay {
                item_type_id: VarInt(5),
            }),
        ]),
    });
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x0a, 0x02, 0x01, 0x04, 0x05]);

    let mut input = encoded.as_slice();
    assert_eq!(SlotDisplay::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn nested_display_variants_are_type_safe() {
    let value = SlotDisplay::OnlyWithComponent(OnlyWithComponentSlotDisplay {
        base: Box::new(SlotDisplay::Empty),
        component_type: DataComponentType::ItemModel,
    });
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0x03, 0x00, 0x07]);
    assert_eq!(SlotDisplay::decode(&mut encoded.as_slice()).unwrap(), value);
}

#[test]
fn unknown_display_type_is_rejected() {
    let error = SlotDisplay::decode(&mut [0x0b].as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 11 })
    );
    assert_eq!(error.codec(), CodecKind::SlotDisplay);
}
