//! Typed payloads shared by debug subscription events and updates.

use std::{fmt, io::Read};

use mcproto_codec::error::{CodecError, CodecKind};

use crate::{
    Boolean, Double, Float, Int, Position, PrefixedArray, PrefixedOptional, PrefixedString,
    ProtocolEnum, TypeCodec, TypeStructCodec, VarInt,
    basic::{decode_prefixed_string, encode_prefixed_string},
};

/// Numeric discriminator for a debug subscription payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum DebugSubscriptionType {
    /// Dedicated server tick-time information with no payload fields.
    DedicatedServerTickTime = 0,
    /// Bee navigation and hive information.
    Bee = 1,
    /// Villager brain state.
    VillagerBrain = 2,
    /// Breeze combat and jump targets.
    Breeze = 3,
    /// One entity goal-selector entry.
    GoalSelector = 4,
    /// One entity path.
    EntityPath = 5,
    /// How an entity intersects its current block or fluid.
    EntityBlockIntersection = 6,
    /// Bee-hive state.
    BeeHive = 7,
    /// Point-of-interest state.
    PointOfInterest = 8,
    /// Redstone-wire orientation.
    RedstoneWireOrientation = 9,
    /// Village-section information with no payload fields.
    VillageSection = 10,
    /// Raid center positions.
    Raid = 11,
    /// Structure bounding boxes and pieces.
    Structure = 12,
    /// Game-event listener radius.
    GameEventListener = 13,
    /// Neighbor-update position.
    NeighborUpdate = 14,
    /// A game event and its world-space coordinates.
    GameEvent = 15,
}

/// Bee navigation and hive debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct BeeDebugData {
    /// Hive position, when one is assigned.
    pub hive_position: PrefixedOptional<Position>,
    /// Flower position, when one is remembered.
    pub flower_position: PrefixedOptional<Position>,
    /// Number of travel ticks.
    pub travel_ticks: VarInt,
    /// Hive positions this bee will not use.
    pub blacklisted_hives: PrefixedArray<Position>,
}

/// Villager brain debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct VillagerBrainDebugData {
    /// Villager name.
    pub name: PrefixedString,
    /// Villager profession name.
    pub profession: PrefixedString,
    /// Villager experience.
    pub xp: Int,
    /// Current health.
    pub health: Float,
    /// Maximum health.
    pub max_health: Float,
    /// Text representation of the inventory.
    pub inventory: PrefixedString,
    /// Whether the villager wants an iron golem.
    pub wants_golem: Boolean,
    /// Current anger level.
    pub anger_level: Int,
    /// Active brain activities.
    pub activities: PrefixedArray<PrefixedString>,
    /// Active behaviors.
    pub behaviors: PrefixedArray<PrefixedString>,
    /// Brain memories.
    pub memories: PrefixedArray<PrefixedString>,
    /// Villager gossips.
    pub gossips: PrefixedArray<PrefixedString>,
    /// Claimed points of interest.
    pub pois: PrefixedArray<Position>,
    /// Candidate points of interest.
    pub potential_pois: PrefixedArray<Position>,
}

/// Breeze target debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct BreezeDebugData {
    /// Optional target entity ID.
    pub attack_target: PrefixedOptional<VarInt>,
    /// Optional jump destination.
    pub jump_target: PrefixedOptional<Position>,
}

/// A goal-selector name limited to 255 UTF-16 code units.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DebugGoalName(String);

impl DebugGoalName {
    /// Maximum number of UTF-16 code units permitted by the protocol.
    pub const MAX_UTF16_CODE_UNITS: usize = 255;
    /// Maximum UTF-8 payload size permitted by the protocol.
    pub const MAX_BYTES: usize = Self::MAX_UTF16_CODE_UNITS * 3;

    /// Creates a validated goal-selector name.
    pub fn new(value: impl Into<String>) -> Result<Self, DebugGoalNameTooLong> {
        let value = value.into();
        let actual_code_units = value.encode_utf16().count();
        if actual_code_units > Self::MAX_UTF16_CODE_UNITS {
            return Err(DebugGoalNameTooLong { actual_code_units });
        }
        Ok(Self(value))
    }

    /// Returns the name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extracts the owned string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for DebugGoalName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DebugGoalName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<String> for DebugGoalName {
    type Error = DebugGoalNameTooLong;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for DebugGoalName {
    type Error = DebugGoalNameTooLong;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<DebugGoalName> for String {
    fn from(value: DebugGoalName) -> Self {
        value.into_inner()
    }
}

impl TypeCodec for DebugGoalName {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        encode_prefixed_string(
            &self.0,
            writer,
            CodecKind::String,
            Self::MAX_BYTES,
            Self::MAX_UTF16_CODE_UNITS,
        )
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        decode_prefixed_string(
            reader,
            CodecKind::String,
            Self::MAX_BYTES,
            Self::MAX_UTF16_CODE_UNITS,
        )
        .map(|(value, _)| Self(value))
    }
}

/// Error returned when a goal-selector name exceeds 255 UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugGoalNameTooLong {
    /// Number of UTF-16 code units in the rejected name.
    pub actual_code_units: usize,
}

impl fmt::Display for DebugGoalNameTooLong {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "debug goal name contains {} UTF-16 code units; maximum is {}",
            self.actual_code_units,
            DebugGoalName::MAX_UTF16_CODE_UNITS
        )
    }
}

impl std::error::Error for DebugGoalNameTooLong {}

/// One goal-selector debug entry.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct GoalSelectorDebugData {
    /// Goal priority.
    pub priority: VarInt,
    /// Whether the goal is currently running.
    pub is_running: Boolean,
    /// Goal name, limited to 255 UTF-16 code units.
    pub name: DebugGoalName,
}

/// Path-node classification used by [`DebugPathNode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum DebugPathNodeType {
    Blocked = 0,
    Open = 1,
    Walkable = 2,
    WalkableDoor = 3,
    Trapdoor = 4,
    PowderSnow = 5,
    DangerPowderSnow = 6,
    Fence = 7,
    Lava = 8,
    Water = 9,
    WaterBorder = 10,
    Rail = 11,
    UnpassableRail = 12,
    DangerFire = 13,
    DamageFire = 14,
    DangerOther = 15,
    DamageOther = 16,
    DoorOpen = 17,
    DoorWoodClosed = 18,
    DoorIronClosed = 19,
    Breach = 20,
    Leaves = 21,
    StickyHoney = 22,
    Cocoa = 23,
    DamageCautious = 24,
    DangerTrapdoor = 25,
}

/// One node in an entity path debug payload.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugPathNode)]
pub struct DebugPathNode {
    pub x: Int,
    pub y: Int,
    pub z: Int,
    pub walked_distance: Float,
    pub cost_malus: Float,
    pub closed: Boolean,
    pub node_type: DebugPathNodeType,
    /// The protocol field named `F`.
    pub f: Float,
}

/// Entity pathfinding debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct EntityPathDebugData {
    pub reached: Boolean,
    pub next_block_index: Int,
    pub block_position: Position,
    pub nodes: PrefixedArray<DebugPathNode>,
    pub target_nodes: PrefixedArray<DebugPathNode>,
    pub open_set: PrefixedArray<DebugPathNode>,
    pub closed_set: PrefixedArray<DebugPathNode>,
    pub max_node_distance: Float,
}

/// Entity/block intersection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum EntityBlockIntersectionState {
    InBlock = 0,
    InFluid = 1,
    InAir = 2,
}

/// Entity/block intersection debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct EntityBlockIntersectionDebugData {
    pub state: EntityBlockIntersectionState,
}

/// Bee-hive debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct BeeHiveDebugData {
    /// ID in the `minecraft:block` registry.
    pub block_type: VarInt,
    pub occupant_count: VarInt,
    pub honey_level: VarInt,
    pub sedated: Boolean,
}

/// Point-of-interest debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct PointOfInterestDebugData {
    pub position: Position,
    /// ID in the `minecraft:point_of_interest_type` registry.
    pub poi_type: VarInt,
    pub free_ticket_count: VarInt,
}

/// Redstone-wire orientation debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct RedstoneWireOrientationDebugData {
    pub id: VarInt,
}

/// Raid debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct RaidDebugData {
    pub positions: PrefixedArray<Position>,
}

/// One structure piece bounding box.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugStructureInfo)]
pub struct DebugStructurePiece {
    pub bounding_box_min: Position,
    pub bounding_box_max: Position,
    pub is_start: Boolean,
}

/// Structure bounding-box debug information.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugStructureInfo)]
pub struct DebugStructureInfo {
    pub bounding_box_min: Position,
    pub bounding_box_max: Position,
    pub pieces: PrefixedArray<DebugStructurePiece>,
}

/// Structure debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct StructureDebugData {
    pub structures: PrefixedArray<DebugStructureInfo>,
}

/// Game-event listener debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct GameEventListenerDebugData {
    pub listener_radius: VarInt,
}

/// Neighbor-update debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct NeighborUpdateDebugData {
    pub position: Position,
}

/// Game-event debug data.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = DebugSubscriptionData)]
pub struct GameEventDebugData {
    /// ID in the `minecraft:game_event` registry.
    pub event: VarInt,
    pub x: Double,
    pub y: Double,
    pub z: Double,
}

/// A payload selected by [`DebugSubscriptionType`].
///
/// The enum variant determines the discriminator written by
/// [`DebugSubscriptionEvent`](crate::DebugSubscriptionEvent) and prevents a
/// payload from being paired with the wrong subscription type.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugSubscriptionData {
    DedicatedServerTickTime,
    Bee(BeeDebugData),
    VillagerBrain(VillagerBrainDebugData),
    Breeze(BreezeDebugData),
    GoalSelector(GoalSelectorDebugData),
    EntityPath(EntityPathDebugData),
    EntityBlockIntersection(EntityBlockIntersectionDebugData),
    BeeHive(BeeHiveDebugData),
    PointOfInterest(PointOfInterestDebugData),
    RedstoneWireOrientation(RedstoneWireOrientationDebugData),
    VillageSection,
    Raid(RaidDebugData),
    Structure(StructureDebugData),
    GameEventListener(GameEventListenerDebugData),
    NeighborUpdate(NeighborUpdateDebugData),
    GameEvent(GameEventDebugData),
}

impl DebugSubscriptionData {
    /// Returns the discriminator associated with this payload.
    #[must_use]
    pub const fn subscription_type(&self) -> DebugSubscriptionType {
        match self {
            Self::DedicatedServerTickTime => DebugSubscriptionType::DedicatedServerTickTime,
            Self::Bee(_) => DebugSubscriptionType::Bee,
            Self::VillagerBrain(_) => DebugSubscriptionType::VillagerBrain,
            Self::Breeze(_) => DebugSubscriptionType::Breeze,
            Self::GoalSelector(_) => DebugSubscriptionType::GoalSelector,
            Self::EntityPath(_) => DebugSubscriptionType::EntityPath,
            Self::EntityBlockIntersection(_) => DebugSubscriptionType::EntityBlockIntersection,
            Self::BeeHive(_) => DebugSubscriptionType::BeeHive,
            Self::PointOfInterest(_) => DebugSubscriptionType::PointOfInterest,
            Self::RedstoneWireOrientation(_) => DebugSubscriptionType::RedstoneWireOrientation,
            Self::VillageSection => DebugSubscriptionType::VillageSection,
            Self::Raid(_) => DebugSubscriptionType::Raid,
            Self::Structure(_) => DebugSubscriptionType::Structure,
            Self::GameEventListener(_) => DebugSubscriptionType::GameEventListener,
            Self::NeighborUpdate(_) => DebugSubscriptionType::NeighborUpdate,
            Self::GameEvent(_) => DebugSubscriptionType::GameEvent,
        }
    }

    pub(crate) fn encode_payload(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), CodecError> {
        match self {
            Self::DedicatedServerTickTime | Self::VillageSection => Ok(()),
            Self::Bee(value) => value.encode(writer),
            Self::VillagerBrain(value) => value.encode(writer),
            Self::Breeze(value) => value.encode(writer),
            Self::GoalSelector(value) => value.encode(writer),
            Self::EntityPath(value) => value.encode(writer),
            Self::EntityBlockIntersection(value) => value.encode(writer),
            Self::BeeHive(value) => value.encode(writer),
            Self::PointOfInterest(value) => value.encode(writer),
            Self::RedstoneWireOrientation(value) => value.encode(writer),
            Self::Raid(value) => value.encode(writer),
            Self::Structure(value) => value.encode(writer),
            Self::GameEventListener(value) => value.encode(writer),
            Self::NeighborUpdate(value) => value.encode(writer),
            Self::GameEvent(value) => value.encode(writer),
        }
    }

    pub(crate) fn decode_payload(
        subscription_type: DebugSubscriptionType,
        reader: &mut impl Read,
    ) -> Result<Self, CodecError> {
        match subscription_type {
            DebugSubscriptionType::DedicatedServerTickTime => Ok(Self::DedicatedServerTickTime),
            DebugSubscriptionType::Bee => BeeDebugData::decode(reader).map(Self::Bee),
            DebugSubscriptionType::VillagerBrain => {
                VillagerBrainDebugData::decode(reader).map(Self::VillagerBrain)
            }
            DebugSubscriptionType::Breeze => BreezeDebugData::decode(reader).map(Self::Breeze),
            DebugSubscriptionType::GoalSelector => {
                GoalSelectorDebugData::decode(reader).map(Self::GoalSelector)
            }
            DebugSubscriptionType::EntityPath => {
                EntityPathDebugData::decode(reader).map(Self::EntityPath)
            }
            DebugSubscriptionType::EntityBlockIntersection => {
                EntityBlockIntersectionDebugData::decode(reader).map(Self::EntityBlockIntersection)
            }
            DebugSubscriptionType::BeeHive => BeeHiveDebugData::decode(reader).map(Self::BeeHive),
            DebugSubscriptionType::PointOfInterest => {
                PointOfInterestDebugData::decode(reader).map(Self::PointOfInterest)
            }
            DebugSubscriptionType::RedstoneWireOrientation => {
                RedstoneWireOrientationDebugData::decode(reader).map(Self::RedstoneWireOrientation)
            }
            DebugSubscriptionType::VillageSection => Ok(Self::VillageSection),
            DebugSubscriptionType::Raid => RaidDebugData::decode(reader).map(Self::Raid),
            DebugSubscriptionType::Structure => {
                StructureDebugData::decode(reader).map(Self::Structure)
            }
            DebugSubscriptionType::GameEventListener => {
                GameEventListenerDebugData::decode(reader).map(Self::GameEventListener)
            }
            DebugSubscriptionType::NeighborUpdate => {
                NeighborUpdateDebugData::decode(reader).map(Self::NeighborUpdate)
            }
            DebugSubscriptionType::GameEvent => {
                GameEventDebugData::decode(reader).map(Self::GameEvent)
            }
        }
    }
}
