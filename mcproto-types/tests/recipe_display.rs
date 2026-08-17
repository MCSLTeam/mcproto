//! Protocol tests for Recipe Display.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    CraftingShapedRecipeDisplay, CraftingShapelessRecipeDisplay, Float, FurnaceRecipeDisplay,
    InvalidShapedRecipeGrid, PrefixedArray, RecipeDisplay, ShapedRecipeGrid, SlotDisplay,
    SmithingRecipeDisplay, StonecutterRecipeDisplay, TypeCodec, VarInt,
};

fn every_recipe_display() -> Vec<RecipeDisplay> {
    vec![
        RecipeDisplay::CraftingShapeless(CraftingShapelessRecipeDisplay {
            ingredients: PrefixedArray(vec![SlotDisplay::Empty, SlotDisplay::AnyFuel]),
            result: SlotDisplay::AnyFuel,
            crafting_station: SlotDisplay::Empty,
        }),
        RecipeDisplay::CraftingShaped(
            CraftingShapedRecipeDisplay::new(
                2,
                1,
                vec![SlotDisplay::Empty, SlotDisplay::AnyFuel],
                SlotDisplay::Empty,
                SlotDisplay::AnyFuel,
            )
            .unwrap(),
        ),
        RecipeDisplay::Furnace(FurnaceRecipeDisplay {
            ingredient: SlotDisplay::Empty,
            fuel: SlotDisplay::AnyFuel,
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::AnyFuel,
            cooking_time: VarInt(200),
            experience: Float(0.5),
        }),
        RecipeDisplay::Stonecutter(StonecutterRecipeDisplay {
            ingredient: SlotDisplay::AnyFuel,
            result: SlotDisplay::Empty,
            crafting_station: SlotDisplay::AnyFuel,
        }),
        RecipeDisplay::Smithing(SmithingRecipeDisplay {
            template: SlotDisplay::Empty,
            base: SlotDisplay::AnyFuel,
            addition: SlotDisplay::Empty,
            result: SlotDisplay::AnyFuel,
            crafting_station: SlotDisplay::Empty,
        }),
    ]
}

#[test]
fn all_five_recipe_display_types_roundtrip() {
    for (expected_type, display) in every_recipe_display().into_iter().enumerate() {
        let mut encoded = Vec::new();
        display.encode(&mut encoded).unwrap();
        assert_eq!(encoded[0], expected_type as u8);

        let mut input = encoded.as_slice();
        assert_eq!(RecipeDisplay::decode(&mut input).unwrap(), display);
        assert!(input.is_empty());
    }
}

#[test]
fn crafting_displays_follow_the_documented_wire_layout() {
    let shapeless = &every_recipe_display()[0];
    let mut encoded = Vec::new();
    shapeless.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0, 2, 0, 1, 1, 0]);

    let shaped = &every_recipe_display()[1];
    let mut encoded = Vec::new();
    shaped.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [1, 2, 1, 2, 0, 1, 0, 1]);
}

#[test]
fn shaped_grid_constructor_enforces_dimensions_and_count() {
    let grid = ShapedRecipeGrid::new(2, 1, vec![SlotDisplay::Empty, SlotDisplay::AnyFuel]).unwrap();
    assert_eq!(grid.dimensions(), (2, 1));
    assert_eq!(grid.ingredients().len(), 2);

    assert_eq!(
        ShapedRecipeGrid::new(2, 2, vec![SlotDisplay::Empty]).unwrap_err(),
        InvalidShapedRecipeGrid::IngredientCountMismatch {
            expected: 4,
            actual: 1,
        }
    );
    assert_eq!(
        ShapedRecipeGrid::new(i32::MAX as u32 + 1, 0, Vec::new()).unwrap_err(),
        InvalidShapedRecipeGrid::WidthOutOfRange {
            width: i32::MAX as u32 + 1,
        }
    );
    assert_eq!(
        ShapedRecipeGrid::new(50_000, 50_000, Vec::new()).unwrap_err(),
        InvalidShapedRecipeGrid::AreaOutOfRange {
            width: 50_000,
            height: 50_000,
        }
    );
}

#[test]
fn decoding_rejects_a_shaped_ingredient_count_mismatch() {
    let error = RecipeDisplay::decode(&mut [1, 2, 2, 3].as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::ShapedRecipeGrid);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::ArrayLengthMismatch {
            expected: 4,
            actual: 3,
        })
    );
    assert_eq!(error.contexts(), &[CodecKind::RecipeDisplay]);
}

#[test]
fn decoding_rejects_unknown_recipe_display_type() {
    let error = RecipeDisplay::decode(&mut [5].as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::RecipeDisplay);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 5 })
    );
}
