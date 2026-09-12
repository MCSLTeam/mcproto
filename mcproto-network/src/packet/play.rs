//! Play state packets for protocol 776 (Minecraft Java Edition 26.2).

use mcproto_types::{Angle, Double, LpVec3, Position, PrefixedArray, ProtocolEnum, TypeStructCodec, UnsignedByte, Uuid, VarInt};

use crate::PacketCodec;

#[derive(PacketCodec)]
#[packet(
    name = "bundle_delimiter",
    id = 0x00,
    state = Play,
    direction = Clientbound,
)]
/// The delimiter for a bundle of packets.
///
/// When received, the client should store every subsequent packet it
/// receives and wait until another delimiter is received. Once that happens,
/// the client is guaranteed to process every packet in the bundle on the
/// same tick, and the client should stop storing packets.
///
/// As of 1.20.6, the vanilla server only uses this to ensure
/// spawned entities and their associated packets are processed on the same
/// tick. Each entity gets a separate bundle.
///
/// The vanilla client doesn't allow more than 4096 packets in a bundle before
/// throwing an error. This packet contains no body bytes of its own.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Bundle_Delimiter)
pub struct BundleDelimiter;

#[derive(PacketCodec)]
#[packet(
    name = "accept_teleportation",
    id = 0x00,
    state = Play,
    direction = Serverbound,
)]
/// Sent by client as confirmation of Synchronize Player Position.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Confirm_Teleportation)
pub struct ConfirmTeleportation {
    /// The ID given by the Synchronize Player Position packet.
    pub teleport_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "add_entity",
    id = 0x01,
    state = Play,
    direction = Clientbound,
)]
/// Sent by the server to create an entity on the client, normally upon the
/// entity spawning within or entering the player's view range.
///
/// The local player entity is automatically created by the client, and must
/// not be created explicitly using this packet. Doing so on the vanilla client
/// will have strange consequences.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Spawn_Entity)
pub struct SpawnEntity {
    /// A unique integer ID mostly used in the protocol to identify the entity.
    /// If an entity with the same ID already exists on the client, it is
    /// automatically deleted and replaced by the new entity. On the vanilla
    /// server entity IDs are globally unique across all dimensions and never
    /// reused while the server is running, but not preserved across server
    /// restarts.
    pub entity_id: VarInt,
    /// A unique identifier that is mostly used in persistence and places
    /// where the uniqueness matters more. It is possible to create multiple
    /// entities with the same UUID on the vanilla client, but a warning will
    /// be logged, and functionality dependent on UUIDs may ignore the entity
    /// or otherwise misbehave.
    pub entity_uuid: Uuid,
    /// ID in the `minecraft:entity_type` registry.
    pub r#type: VarInt,
    /// Position X.
    pub x: Double,
    /// Position Y.
    pub y: Double,
    /// Position Z.
    pub z: Double,
    /// Velocity of the entity in the X, Y, and Z directions. The velocity is measured in.
    pub velocity: LpVec3,
    /// Pitch of the entity in degrees.
    pub pitch: Angle,
    /// Yaw of the entity in degrees.
    pub yaw: Angle,
    /// Only used by living entities, where the head of the entity may differ
    /// from the general body rotation.
    pub head_yaw: Angle,
    /// Meaning dependent on the value of the Type field.
    pub data: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "attack",
    id = 0x01,
    state = Play,
    direction = Serverbound,
)]
/// Sent from the client to the server when the client attacks another entity
/// (a player, minecart, etc).
///
/// A vanilla server only accepts this packet if the entity being attacked is
/// visible without obstruction and within a 4-unit radius of the player's
/// position.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Attack)
pub struct Attack {
    /// The ID of the attacked entity.
    ///
    /// Note the special case of the ender dragon described on the Interact
    /// packet.
    pub entity_id: VarInt,
}

/// Animation ID that selects which animation should be played on an entity.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Animation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = UnsignedByte)]
pub enum EntityAnimationKind {
    /// Swing the player's main arm.
    SwingMainArm = 0,
    /// Leave the bed animation.
    LeaveBed = 2,
    /// Swing the player's offhand.
    SwingOffhand = 3,
    /// Critical hit visual effect.
    CriticalEffect = 4,
    /// Magic enchant critical hit visual effect.
    MagicCriticalEffect = 5,
}

#[derive(PacketCodec)]
#[packet(
    name = "animate",
    id = 0x02,
    state = Play,
    direction = Clientbound,
)]
/// Sent whenever an entity should change animation.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Animation)
pub struct EntityAnimation {
    /// Entity identifier (player ID).
    pub entity_id: VarInt,
    /// Which animation to trigger on the entity.
    pub animation: EntityAnimationKind,
}
#[derive(PacketCodec)]
#[packet(
    name = "block_entity_tag_query",
    id = 0x02,
    state = Play,
    direction = Serverbound,
)]
/// Used when F3+I is pressed while looking at a block.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Query_Block_Entity_Tag)
pub struct QueryBlockEntityTag {
    /// An incremental ID so that the client can verify that the response matches.
    pub transaction_id: VarInt,
    /// The location of the block to check.
    pub location: Position,
}
/// One individual statistic entry inside the Award Statistics packet.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct StatisticEntry {
    /// ID in the `minecraft:stat_type` registry.
    pub category_id: VarInt,
    /// The statistic ID inside the selected category.
    pub statistic_id: VarInt,
    /// The new numeric value for this statistic.
    pub value: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "award_stats",
    id = 0x03,
    state = Play,
    direction = Clientbound,
)]
/// Sent as a response to Client Status (id 1). Will only send the changed
/// values if previously requested.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Award_Statistics)
pub struct AwardStatistics {
    /// List of updated statistic entries.
    pub statistics: PrefixedArray<StatisticEntry>,
}
#[derive(PacketCodec)]
#[packet(
    name = "bundle_item_selected",
    id = 0x03,
    state = Play,
    direction = Serverbound,
)]
/// Sent by the client when the player selects an item from inside the
/// bundle item tooltip preview.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Bundle_Item_Selected)
pub struct BundleItemSelected {
    /// Slot index of the bundle container item in the player's inventory.
    pub bundle_slot: VarInt,
    /// Index of the item inside that bundle.
    pub selected_slot: VarInt,
}