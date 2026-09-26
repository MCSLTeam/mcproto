//! Command graph node format.
//!
//! The command graph sent by the `Commands` packet is a directed graph made of
//! `root`, `literal`, and `argument` nodes.
//!
//! Parser IDs follow the protocol 777 (Minecraft Java Edition 26.3) table.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind, InvalidEncodingReason};

use crate::contextual::Context;
use crate::{
    Byte, ContextualCodec, Double, Float, Identifier, Int, Long, Optional, PrefixedArray,
    PrefixedString, ProtocolEnum, TypeCodec, UnsignedByte, VarInt,
};

/// Type of a command graph node.
///
/// The type is stored in the low two bits of a node's flags byte. The value
/// `3` is not used by the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandNodeType {
    /// The nameless root node of the graph.
    Root = 0,
    /// A literal such as `msg` or `me`.
    Literal = 1,
    /// An argument parsed by a Brigadier parser.
    Argument = 2,
}

/// Bit mask of command node flags.
///
/// Bits `0x01` and `0x02` together hold the [`CommandNodeType`]. Unknown bits
/// are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandNodeFlags(pub u8);

impl CommandNodeFlags {
    /// Mask selecting the node type in the low two bits.
    pub const TYPE_MASK: u8 = 0x03;
    /// Set if the node stack to this point constitutes a valid command.
    pub const IS_EXECUTABLE: Self = Self(0x04);
    /// Set if the node redirects to another node.
    pub const HAS_REDIRECT: Self = Self(0x08);
    /// Set if the node carries a suggestions type.
    pub const HAS_SUGGESTIONS_TYPE: Self = Self(0x10);
    /// Set if the node requires a player permission level above 0.
    pub const IS_RESTRICTED: Self = Self(0x20);

    /// Returns the raw protocol bit mask.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Returns whether every bit in `mask` is set.
    #[must_use]
    pub const fn contains(self, mask: Self) -> bool {
        self.0 & mask.0 == mask.0
    }

    /// Returns the node type encoded in the flag byte.
    ///
    /// The reserved value `3` is reported as [`CommandNodeType::Argument`].
    #[must_use]
    pub const fn node_type(self) -> CommandNodeType {
        match self.0 & Self::TYPE_MASK {
            0 => CommandNodeType::Root,
            1 => CommandNodeType::Literal,
            _ => CommandNodeType::Argument,
        }
    }
}

impl TypeCodec for CommandNodeFlags {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        UnsignedByte(self.0).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        UnsignedByte::decode(reader).map(|value| Self(value.0))
    }
}

/// Parsing behavior of the `brigadier:string` parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum CommandStringBehavior {
    /// Reads a single word.
    SingleWord = 0,
    /// If it starts with a `"`, keeps reading until another `"` (allowing
    /// escaping with `\`). Otherwise behaves the same as
    /// [`SingleWord`](Self::SingleWord).
    QuotablePhrase = 1,
    /// Reads the rest of the content after the cursor. Quotes will not be
    /// removed.
    GreedyPhrase = 2,
}

/// Bit mask of the `minecraft:entity` parser's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandEntityFlags(pub u8);

impl CommandEntityFlags {
    /// If set, only allows a single entity/player.
    pub const SINGLE: Self = Self(0x01);
    /// If set, only allows players.
    pub const PLAYERS_ONLY: Self = Self(0x02);

    /// Returns the raw protocol bit mask.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Returns whether every bit in `mask` is set.
    #[must_use]
    pub const fn contains(self, mask: Self) -> bool {
        self.0 & mask.0 == mask.0
    }
}

impl TypeCodec for CommandEntityFlags {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        UnsignedByte(self.0).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        UnsignedByte::decode(reader).map(|value| Self(value.0))
    }
}

/// Bit mask of the `minecraft:score_holder` parser's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CommandScoreHolderFlags(pub u8);

impl CommandScoreHolderFlags {
    /// If set, allows multiple.
    pub const ALLOW_MULTIPLE: Self = Self(0x01);

    /// Returns the raw protocol bit mask.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Returns whether every bit in `mask` is set.
    #[must_use]
    pub const fn contains(self, mask: Self) -> bool {
        self.0 & mask.0 == mask.0
    }
}

impl TypeCodec for CommandScoreHolderFlags {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        UnsignedByte(self.0).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        UnsignedByte::decode(reader).map(|value| Self(value.0))
    }
}

/// Brigadier parser of an `argument` command node.
///
/// The numeric parser ID is derived from the variant, so an ID and an
/// incompatible set of properties cannot be paired. Parser IDs follow the
/// protocol 777 (Minecraft Java Edition 26.3) table.
///
/// Range bounds are stored as they appear on the wire. The defaults documented
/// by the protocol apply only while parsing command input, not to the encoded
/// value, so absent bounds stay `None`.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandParser {
    /// `brigadier:bool`.
    Bool,
    /// `brigadier:float`.
    Float {
        /// Lower bound, present when the properties flags include `0x01`.
        min: Option<Float>,
        /// Upper bound, present when the properties flags include `0x02`.
        max: Option<Float>,
    },
    /// `brigadier:double`.
    Double {
        /// Lower bound, present when the properties flags include `0x01`.
        min: Option<Double>,
        /// Upper bound, present when the properties flags include `0x02`.
        max: Option<Double>,
    },
    /// `brigadier:integer`.
    Integer {
        /// Lower bound, present when the properties flags include `0x01`.
        min: Option<Int>,
        /// Upper bound, present when the properties flags include `0x02`.
        max: Option<Int>,
    },
    /// `brigadier:long`.
    Long {
        /// Lower bound, present when the properties flags include `0x01`.
        min: Option<Long>,
        /// Upper bound, present when the properties flags include `0x02`.
        max: Option<Long>,
    },
    /// `brigadier:string`.
    String {
        /// Parsing behavior.
        behavior: CommandStringBehavior,
    },
    /// `minecraft:entity`.
    Entity {
        /// Constraint flags.
        flags: CommandEntityFlags,
    },
    /// `minecraft:game_profile`.
    GameProfile,
    /// `minecraft:block_pos`.
    BlockPos,
    /// `minecraft:column_pos`.
    ColumnPos,
    /// `minecraft:vec3`.
    Vec3,
    /// `minecraft:vec2`.
    Vec2,
    /// `minecraft:block_state`.
    BlockState,
    /// `minecraft:block_predicate`.
    BlockPredicate,
    /// `minecraft:item_stack`.
    ItemStack,
    /// `minecraft:item_predicate`.
    ItemPredicate,
    /// `minecraft:color`.
    Color,
    /// `minecraft:hex_color`.
    HexColor,
    /// `minecraft:component`.
    Component,
    /// `minecraft:style`.
    Style,
    /// `minecraft:message`.
    Message,
    /// `minecraft:nbt_compound_tag`.
    NbtCompoundTag,
    /// `minecraft:nbt_tag`.
    NbtTag,
    /// `minecraft:nbt_path`.
    NbtPath,
    /// `minecraft:objective`.
    Objective,
    /// `minecraft:objective_criteria`.
    ObjectiveCriteria,
    /// `minecraft:operation`.
    Operation,
    /// `minecraft:particle`.
    Particle,
    /// `minecraft:angle`.
    Angle,
    /// `minecraft:rotation`.
    Rotation,
    /// `minecraft:scoreboard_slot`.
    ScoreboardSlot,
    /// `minecraft:score_holder`.
    ScoreHolder {
        /// Constraint flags.
        flags: CommandScoreHolderFlags,
    },
    /// `minecraft:swizzle`.
    Swizzle,
    /// `minecraft:team`.
    Team,
    /// `minecraft:item_slot`.
    ItemSlot,
    /// `minecraft:item_slots`.
    ItemSlots,
    /// `minecraft:resource_location`.
    ResourceLocation,
    /// `minecraft:function`.
    Function,
    /// `minecraft:entity_anchor`.
    EntityAnchor,
    /// `minecraft:int_range`.
    IntRange,
    /// `minecraft:float_range`.
    FloatRange,
    /// `minecraft:dimension`.
    Dimension,
    /// `minecraft:gamemode`.
    GameMode,
    /// `minecraft:time`.
    Time {
        /// Minimum duration in ticks.
        min: Int,
    },
    /// `minecraft:resource_or_tag`.
    ResourceOrTag {
        /// Registry supplying suggestions.
        registry: Identifier,
    },
    /// `minecraft:resource_or_tag_key`.
    ResourceOrTagKey {
        /// Registry supplying suggestions.
        registry: Identifier,
    },
    /// `minecraft:resource`.
    Resource {
        /// Registry supplying suggestions.
        registry: Identifier,
    },
    /// `minecraft:resource_key`.
    ResourceKey {
        /// Registry supplying suggestions.
        registry: Identifier,
    },
    /// `minecraft:resource_selector`.
    ResourceSelector {
        /// Registry supplying suggestions.
        registry: Identifier,
    },
    /// `minecraft:template_mirror`.
    TemplateMirror,
    /// `minecraft:template_rotation`.
    TemplateRotation,
    /// `minecraft:heightmap`.
    Heightmap,
    /// `minecraft:loot_table`.
    LootTable,
    /// `minecraft:loot_predicate`.
    LootPredicate,
    /// `minecraft:loot_modifier`.
    LootModifier,
    /// `minecraft:dialog`.
    Dialog,
    /// `minecraft:uuid`.
    Uuid,
}

impl CommandParser {
    /// Returns the numeric parser ID written to the wire.
    #[must_use]
    pub const fn parser_id(&self) -> i32 {
        match self {
            Self::Bool => 0,
            Self::Float { .. } => 1,
            Self::Double { .. } => 2,
            Self::Integer { .. } => 3,
            Self::Long { .. } => 4,
            Self::String { .. } => 5,
            Self::Entity { .. } => 6,
            Self::GameProfile => 7,
            Self::BlockPos => 8,
            Self::ColumnPos => 9,
            Self::Vec3 => 10,
            Self::Vec2 => 11,
            Self::BlockState => 12,
            Self::BlockPredicate => 13,
            Self::ItemStack => 14,
            Self::ItemPredicate => 15,
            Self::Color => 16,
            Self::HexColor => 17,
            Self::Component => 18,
            Self::Style => 19,
            Self::Message => 20,
            Self::NbtCompoundTag => 21,
            Self::NbtTag => 22,
            Self::NbtPath => 23,
            Self::Objective => 24,
            Self::ObjectiveCriteria => 25,
            Self::Operation => 26,
            Self::Particle => 27,
            Self::Angle => 28,
            Self::Rotation => 29,
            Self::ScoreboardSlot => 30,
            Self::ScoreHolder { .. } => 31,
            Self::Swizzle => 32,
            Self::Team => 33,
            Self::ItemSlot => 34,
            Self::ItemSlots => 35,
            Self::ResourceLocation => 36,
            Self::Function => 37,
            Self::EntityAnchor => 38,
            Self::IntRange => 39,
            Self::FloatRange => 40,
            Self::Dimension => 41,
            Self::GameMode => 42,
            Self::Time { .. } => 43,
            Self::ResourceOrTag { .. } => 44,
            Self::ResourceOrTagKey { .. } => 45,
            Self::Resource { .. } => 46,
            Self::ResourceKey { .. } => 47,
            Self::ResourceSelector { .. } => 48,
            Self::TemplateMirror => 49,
            Self::TemplateRotation => 50,
            Self::Heightmap => 51,
            Self::LootTable => 52,
            Self::LootPredicate => 53,
            Self::LootModifier => 54,
            Self::Dialog => 55,
            Self::Uuid => 56,
        }
    }
}

/// Encodes a range parser's flags byte followed by its optional bounds.
macro_rules! encode_bounds {
    ($writer:expr, $min:expr, $max:expr) => {{
        let min = $min;
        let max = $max;
        let mut flags = 0u8;
        if min.is_some() {
            flags |= 0x01;
        }
        if max.is_some() {
            flags |= 0x02;
        }
        Byte(flags as i8).encode($writer)?;
        if let Some(value) = min {
            value.encode($writer)?;
        }
        if let Some(value) = max {
            value.encode($writer)?;
        }
    }};
}

/// Decodes a range parser's flags byte and its optional bounds.
macro_rules! decode_bounds {
    ($reader:expr, $ty:ty) => {{
        let flags = Byte::decode($reader)?.0 as u8;
        let min = if flags & 0x01 != 0 {
            Some(<$ty>::decode($reader)?)
        } else {
            None
        };
        let max = if flags & 0x02 != 0 {
            Some(<$ty>::decode($reader)?)
        } else {
            None
        };
        (min, max)
    }};
}

impl TypeCodec for CommandParser {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        VarInt(self.parser_id()).encode(writer)?;

        match self {
            Self::Float { min, max } => encode_bounds!(writer, min.as_ref(), max.as_ref()),
            Self::Double { min, max } => encode_bounds!(writer, min.as_ref(), max.as_ref()),
            Self::Integer { min, max } => encode_bounds!(writer, min.as_ref(), max.as_ref()),
            Self::Long { min, max } => encode_bounds!(writer, min.as_ref(), max.as_ref()),
            Self::String { behavior } => behavior.encode(writer)?,
            Self::Entity { flags } => flags.encode(writer)?,
            Self::ScoreHolder { flags } => flags.encode(writer)?,
            Self::Time { min } => min.encode(writer)?,
            Self::ResourceOrTag { registry }
            | Self::ResourceOrTagKey { registry }
            | Self::Resource { registry }
            | Self::ResourceKey { registry }
            | Self::ResourceSelector { registry } => registry.encode(writer)?,
            _ => {}
        }

        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let id = VarInt::decode(reader)?;

        Ok(match id.0 {
            0 => Self::Bool,
            1 => {
                let (min, max) = decode_bounds!(reader, Float);
                Self::Float { min, max }
            }
            2 => {
                let (min, max) = decode_bounds!(reader, Double);
                Self::Double { min, max }
            }
            3 => {
                let (min, max) = decode_bounds!(reader, Int);
                Self::Integer { min, max }
            }
            4 => {
                let (min, max) = decode_bounds!(reader, Long);
                Self::Long { min, max }
            }
            5 => Self::String {
                behavior: CommandStringBehavior::decode(reader)?,
            },
            6 => Self::Entity {
                flags: CommandEntityFlags::decode(reader)?,
            },
            7 => Self::GameProfile,
            8 => Self::BlockPos,
            9 => Self::ColumnPos,
            10 => Self::Vec3,
            11 => Self::Vec2,
            12 => Self::BlockState,
            13 => Self::BlockPredicate,
            14 => Self::ItemStack,
            15 => Self::ItemPredicate,
            16 => Self::Color,
            17 => Self::HexColor,
            18 => Self::Component,
            19 => Self::Style,
            20 => Self::Message,
            21 => Self::NbtCompoundTag,
            22 => Self::NbtTag,
            23 => Self::NbtPath,
            24 => Self::Objective,
            25 => Self::ObjectiveCriteria,
            26 => Self::Operation,
            27 => Self::Particle,
            28 => Self::Angle,
            29 => Self::Rotation,
            30 => Self::ScoreboardSlot,
            31 => Self::ScoreHolder {
                flags: CommandScoreHolderFlags::decode(reader)?,
            },
            32 => Self::Swizzle,
            33 => Self::Team,
            34 => Self::ItemSlot,
            35 => Self::ItemSlots,
            36 => Self::ResourceLocation,
            37 => Self::Function,
            38 => Self::EntityAnchor,
            39 => Self::IntRange,
            40 => Self::FloatRange,
            41 => Self::Dimension,
            42 => Self::GameMode,
            43 => Self::Time {
                min: Int::decode(reader)?,
            },
            44 => Self::ResourceOrTag {
                registry: Identifier::decode(reader)?,
            },
            45 => Self::ResourceOrTagKey {
                registry: Identifier::decode(reader)?,
            },
            46 => Self::Resource {
                registry: Identifier::decode(reader)?,
            },
            47 => Self::ResourceKey {
                registry: Identifier::decode(reader)?,
            },
            48 => Self::ResourceSelector {
                registry: Identifier::decode(reader)?,
            },
            49 => Self::TemplateMirror,
            50 => Self::TemplateRotation,
            51 => Self::Heightmap,
            52 => Self::LootTable,
            53 => Self::LootPredicate,
            54 => Self::LootModifier,
            55 => Self::Dialog,
            56 => Self::Uuid,
            other => {
                return Err(CodecError::invalid_encoding(
                    CodecKind::Enum,
                    0,
                    InvalidEncodingReason::InvalidEnumValue {
                        value: other as i128,
                    },
                ));
            }
        })
    }
}

/// One node of the command graph.
///
/// Fields are present on the wire depending on the node's flags, following the
/// protocol's node format. The name is present for `literal` and `argument`
/// nodes, the parser for `argument` nodes, and the redirect node and
/// suggestions type according to their flags.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandNode {
    /// Flags describing the node's type and capabilities.
    pub flags: CommandNodeFlags,
    /// Indices of child nodes.
    pub children: PrefixedArray<VarInt>,
    /// Index of the redirect node, when the node has one.
    pub redirect_node: Optional<VarInt>,
    /// Name of a `literal` or `argument` node.
    pub name: Optional<PrefixedString>,
    /// Brigadier parser of an `argument` node, including its properties.
    pub parser: Optional<CommandParser>,
    /// Suggestions provider for the node, when it has one.
    pub suggestions_type: Optional<Identifier>,
}

impl CommandNode {
    /// Returns the node type encoded in this node's flags.
    #[must_use]
    pub const fn node_type(&self) -> CommandNodeType {
        self.flags.node_type()
    }
}

impl TypeCodec for CommandNode {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let node_type = self.node_type();
        let has_redirect = self.flags.contains(CommandNodeFlags::HAS_REDIRECT);
        let has_suggestions = self.flags.contains(CommandNodeFlags::HAS_SUGGESTIONS_TYPE);

        self.flags.encode(writer)?;
        self.children.encode(writer)?;
        self.redirect_node
            .encode_with_context(writer, &Context::new(has_redirect))?;
        self.name
            .encode_with_context(writer, &Context::new(node_type != CommandNodeType::Root))?;
        self.parser.encode_with_context(
            writer,
            &Context::new(node_type == CommandNodeType::Argument),
        )?;
        self.suggestions_type
            .encode_with_context(writer, &Context::new(has_suggestions))?;
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let flags = CommandNodeFlags::decode(reader)?;
        let node_type = flags.node_type();
        let has_redirect = flags.contains(CommandNodeFlags::HAS_REDIRECT);
        let has_suggestions = flags.contains(CommandNodeFlags::HAS_SUGGESTIONS_TYPE);

        Ok(Self {
            flags,
            children: PrefixedArray::decode(reader)?,
            redirect_node: Optional::decode_with_context(reader, &Context::new(has_redirect))?,
            name: Optional::decode_with_context(
                reader,
                &Context::new(node_type != CommandNodeType::Root),
            )?,
            parser: Optional::decode_with_context(
                reader,
                &Context::new(node_type == CommandNodeType::Argument),
            )?,
            suggestions_type: Optional::decode_with_context(
                reader,
                &Context::new(has_suggestions),
            )?,
        })
    }
}
