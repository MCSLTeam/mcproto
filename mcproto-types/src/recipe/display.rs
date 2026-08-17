//! Type-safe client recipe displays.

use std::{fmt, io::Read};

use mcproto_codec::error::{CodecError, CodecKind, InvalidEncodingReason};

use crate::{Float, PrefixedArray, SlotDisplay, TypeCodec, TypeStructCodec, VarInt};

/// A shapeless crafting recipe display.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = RecipeDisplay)]
pub struct CraftingShapelessRecipeDisplay {
    /// Ingredient displays. The array prefix is the official ingredient count.
    pub ingredients: PrefixedArray<SlotDisplay>,
    /// Display for the crafted result.
    pub result: SlotDisplay,
    /// Crafting-station icon shown by the client.
    pub crafting_station: SlotDisplay,
}

/// A width, height, and exactly `width * height` ingredient displays.
///
/// Fields are private so the length invariant cannot be broken after
/// construction. Width, height, and ingredient count are encoded as VarInts.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRecipeGrid {
    width: u32,
    height: u32,
    ingredients: Vec<SlotDisplay>,
}

impl ShapedRecipeGrid {
    /// Creates a grid when its dimensions and ingredient count are valid.
    pub fn new(
        width: u32,
        height: u32,
        ingredients: Vec<SlotDisplay>,
    ) -> Result<Self, InvalidShapedRecipeGrid> {
        validate_grid(width, height, ingredients.len())?;
        Ok(Self {
            width,
            height,
            ingredients,
        })
    }

    /// Returns the grid width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the grid height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the grid dimensions as `(width, height)`.
    #[must_use]
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the ingredient displays in row-major order.
    #[must_use]
    pub fn ingredients(&self) -> &[SlotDisplay] {
        &self.ingredients
    }

    /// Extracts the ingredient displays in row-major order.
    #[must_use]
    pub fn into_ingredients(self) -> Vec<SlotDisplay> {
        self.ingredients
    }
}

impl TypeCodec for ShapedRecipeGrid {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        VarInt(self.width as i32)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?;
        VarInt(self.height as i32)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?;
        VarInt(self.ingredients.len() as i32)
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?;
        for ingredient in &self.ingredients {
            ingredient
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let width = decode_grid_dimension(reader)?;
        let height = decode_grid_dimension(reader)?;
        let ingredient_count = decode_ingredient_count(reader)?;
        validate_decoded_grid(width, height, ingredient_count)?;

        let mut ingredients = Vec::new();
        for _ in 0..ingredient_count {
            ingredients.push(
                SlotDisplay::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?,
            );
        }
        Ok(Self {
            width,
            height,
            ingredients,
        })
    }
}

/// A shaped crafting recipe display.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = RecipeDisplay)]
pub struct CraftingShapedRecipeDisplay {
    /// Validated rectangular ingredient grid.
    pub grid: ShapedRecipeGrid,
    /// Display for the crafted result.
    pub result: SlotDisplay,
    /// Crafting-station icon shown by the client.
    pub crafting_station: SlotDisplay,
}

impl CraftingShapedRecipeDisplay {
    /// Creates a shaped display while enforcing the ingredient-grid invariant.
    pub fn new(
        width: u32,
        height: u32,
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    ) -> Result<Self, InvalidShapedRecipeGrid> {
        Ok(Self {
            grid: ShapedRecipeGrid::new(width, height, ingredients)?,
            result,
            crafting_station,
        })
    }
}

/// A furnace-style recipe display.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = RecipeDisplay)]
pub struct FurnaceRecipeDisplay {
    /// Ingredient accepted by the furnace recipe.
    pub ingredient: SlotDisplay,
    /// Fuel display.
    pub fuel: SlotDisplay,
    /// Smelting result display.
    pub result: SlotDisplay,
    /// Furnace icon shown by the client.
    pub crafting_station: SlotDisplay,
    /// Cooking duration in ticks.
    pub cooking_time: VarInt,
    /// Experience awarded by the recipe.
    pub experience: Float,
}

/// A stonecutter recipe display.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = RecipeDisplay)]
pub struct StonecutterRecipeDisplay {
    /// Input ingredient display.
    pub ingredient: SlotDisplay,
    /// Result display.
    pub result: SlotDisplay,
    /// Stonecutter icon shown by the client.
    pub crafting_station: SlotDisplay,
}

/// A smithing recipe display.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = RecipeDisplay)]
pub struct SmithingRecipeDisplay {
    /// Smithing template display.
    pub template: SlotDisplay,
    /// Base item display.
    pub base: SlotDisplay,
    /// Addition material display.
    pub addition: SlotDisplay,
    /// Smithing result display.
    pub result: SlotDisplay,
    /// Smithing-table icon shown by the client.
    pub crafting_station: SlotDisplay,
}

/// A recipe description sent for display by the client.
///
/// Each enum variant fixes both the ID in the `minecraft:recipe_display`
/// registry and its payload structure, so mismatched IDs and payloads cannot
/// be represented. The current protocol IDs are:
///
/// - `0`: `minecraft:crafting_shapeless`
/// - `1`: `minecraft:crafting_shaped`
/// - `2`: `minecraft:furnace`
/// - `3`: `minecraft:stonecutter`
/// - `4`: `minecraft:smithing`
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     CraftingShapedRecipeDisplay, RecipeDisplay, SlotDisplay, TypeCodec,
/// };
///
/// let display = RecipeDisplay::CraftingShaped(
///     CraftingShapedRecipeDisplay::new(
///         2,
///         1,
///         vec![SlotDisplay::Empty, SlotDisplay::AnyFuel],
///         SlotDisplay::Empty,
///         SlotDisplay::AnyFuel,
///     )?,
/// );
/// let mut encoded = Vec::new();
/// display.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(RecipeDisplay::decode(&mut input)?, display);
/// assert!(input.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// See the official [Recipe Display structure] documentation.
///
/// [Recipe Display structure]: https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Recipe_Display_structure
#[derive(Debug, Clone, PartialEq)]
pub enum RecipeDisplay {
    CraftingShapeless(CraftingShapelessRecipeDisplay),
    CraftingShaped(CraftingShapedRecipeDisplay),
    Furnace(FurnaceRecipeDisplay),
    Stonecutter(StonecutterRecipeDisplay),
    Smithing(SmithingRecipeDisplay),
}

impl TypeCodec for RecipeDisplay {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        match self {
            Self::CraftingShapeless(value) => {
                encode_display_type(0, writer)?;
                value.encode(writer)
            }
            Self::CraftingShaped(value) => {
                encode_display_type(1, writer)?;
                value.encode(writer)
            }
            Self::Furnace(value) => {
                encode_display_type(2, writer)?;
                value.encode(writer)
            }
            Self::Stonecutter(value) => {
                encode_display_type(3, writer)?;
                value.encode(writer)
            }
            Self::Smithing(value) => {
                encode_display_type(4, writer)?;
                value.encode(writer)
            }
        }
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let display_type = VarInt::decode(reader)
            .map_err(|error| error.with_context(CodecKind::RecipeDisplay))?
            .0;
        match display_type {
            0 => CraftingShapelessRecipeDisplay::decode(reader).map(Self::CraftingShapeless),
            1 => CraftingShapedRecipeDisplay::decode(reader).map(Self::CraftingShaped),
            2 => FurnaceRecipeDisplay::decode(reader).map(Self::Furnace),
            3 => StonecutterRecipeDisplay::decode(reader).map(Self::Stonecutter),
            4 => SmithingRecipeDisplay::decode(reader).map(Self::Smithing),
            value => Err(CodecError::invalid_encoding(
                CodecKind::RecipeDisplay,
                0,
                InvalidEncodingReason::InvalidEnumValue {
                    value: i128::from(value),
                },
            )),
        }
    }
}

fn encode_display_type(
    display_type: i32,
    writer: &mut impl std::io::Write,
) -> Result<(), CodecError> {
    VarInt(display_type)
        .encode(writer)
        .map_err(|error| error.with_context(CodecKind::RecipeDisplay))
}

fn decode_grid_dimension(reader: &mut impl Read) -> Result<u32, CodecError> {
    let value = VarInt::decode(reader)
        .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?
        .0;
    u32::try_from(value).map_err(|_| {
        CodecError::invalid_encoding(
            CodecKind::ShapedRecipeGrid,
            0,
            InvalidEncodingReason::NegativeLength { value },
        )
    })
}

fn decode_ingredient_count(reader: &mut impl Read) -> Result<usize, CodecError> {
    let value = VarInt::decode(reader)
        .map_err(|error| error.with_context(CodecKind::ShapedRecipeGrid))?
        .0;
    usize::try_from(value).map_err(|_| {
        CodecError::invalid_encoding(
            CodecKind::ShapedRecipeGrid,
            0,
            InvalidEncodingReason::NegativeLength { value },
        )
    })
}

fn validate_decoded_grid(
    width: u32,
    height: u32,
    ingredient_count: usize,
) -> Result<(), CodecError> {
    let expected = encoded_grid_area(width, height).ok_or_else(|| {
        CodecError::invalid_encoding(
            CodecKind::ShapedRecipeGrid,
            0,
            InvalidEncodingReason::LengthOutOfRange {
                max: i32::MAX as usize,
                actual: usize::MAX,
            },
        )
    })?;
    if ingredient_count != expected {
        return Err(CodecError::invalid_encoding(
            CodecKind::ShapedRecipeGrid,
            0,
            InvalidEncodingReason::ArrayLengthMismatch {
                expected,
                actual: ingredient_count,
            },
        ));
    }
    Ok(())
}

fn validate_grid(
    width: u32,
    height: u32,
    ingredient_count: usize,
) -> Result<(), InvalidShapedRecipeGrid> {
    if width > i32::MAX as u32 {
        return Err(InvalidShapedRecipeGrid::WidthOutOfRange { width });
    }
    if height > i32::MAX as u32 {
        return Err(InvalidShapedRecipeGrid::HeightOutOfRange { height });
    }
    let expected = encoded_grid_area(width, height)
        .ok_or(InvalidShapedRecipeGrid::AreaOutOfRange { width, height })?;
    if ingredient_count != expected {
        return Err(InvalidShapedRecipeGrid::IngredientCountMismatch {
            expected,
            actual: ingredient_count,
        });
    }
    Ok(())
}

fn encoded_grid_area(width: u32, height: u32) -> Option<usize> {
    let area = u64::from(width).checked_mul(u64::from(height))?;
    if area > i32::MAX as u64 {
        return None;
    }
    Some(area as usize)
}

/// Error returned when constructing an invalid shaped recipe grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidShapedRecipeGrid {
    /// Width cannot be represented by the protocol VarInt.
    WidthOutOfRange { width: u32 },
    /// Height cannot be represented by the protocol VarInt.
    HeightOutOfRange { height: u32 },
    /// The rectangular area cannot be represented by the ingredient-count VarInt.
    AreaOutOfRange { width: u32, height: u32 },
    /// The supplied ingredient count is not exactly `width * height`.
    IngredientCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for InvalidShapedRecipeGrid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WidthOutOfRange { width } => {
                write!(
                    formatter,
                    "recipe grid width exceeds a positive VarInt: {width}"
                )
            }
            Self::HeightOutOfRange { height } => {
                write!(
                    formatter,
                    "recipe grid height exceeds a positive VarInt: {height}"
                )
            }
            Self::AreaOutOfRange { width, height } => write!(
                formatter,
                "recipe grid area {width} * {height} exceeds a positive VarInt"
            ),
            Self::IngredientCountMismatch { expected, actual } => write!(
                formatter,
                "recipe grid requires {expected} ingredients, got {actual}"
            ),
        }
    }
}

impl std::error::Error for InvalidShapedRecipeGrid {}
