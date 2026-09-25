//! Play state packets for protocol 777 (Minecraft Java Edition 26.3).

pub mod bossbar;
pub mod chunk;

pub use bossbar::*;
pub use chunk::*;

use mcproto_types::{
    Angle, Boolean, BoundedPrefixedArray, BoundedString, Byte, Double, FixedBitSet, FixedByteArray,
    Float, Long, LpVec3, Nbt, Position, PrefixedArray, PrefixedOptional, PrefixedString,
    ProtocolEnum, TypeStructCodec, UnsignedByte, Uuid, VarInt,
};

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
    /// Resulting X position.
    pub x: Double,
    /// Resulting Y position.
    pub y: Double,
    /// Resulting Z position.
    pub z: Double,
    /// Resulting yaw.
    pub yaw: Float,
    /// Resulting pitch.
    pub pitch: Float,
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
    /// Wake up / leave bed.
    WakeUpLeaveBed = 0,
    /// Critical effect.
    CriticalEffect = 1,
    /// Magic critical effect.
    MagicCriticalEffect = 2,
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

#[derive(PacketCodec)]
#[packet(
    name = "block_changed_ack",
    id = 0x04,
    state = Play,
    direction = Clientbound,
)]
/// Acknowledges a user-initiated block change. After receiving this packet, the client will display the block state sent by the server instead of the one predicted by the client.
///
/// Clientbound `block_changed_ack`, Play ID: 4 (0x4)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Block_Change)
pub struct AcknowledgeBlockChange {
    /// Represents the sequence to acknowledge; this is used for properly syncing block changes to the client after interactions.
    pub sequence_id: VarInt,
}

/// Difficulty of the game.
///
/// 0: peaceful, 1: easy, 2: normal, 3: hard.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Difficulty)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = UnsignedByte)]
pub enum Difficulty {
    /// Peaceful.
    Peaceful = 0,
    /// Easy.
    Easy = 1,
    /// Normal.
    Normal = 2,
    /// Hard.
    Hard = 3,
}

#[derive(PacketCodec)]
#[packet(
    name = "change_difficulty",
    id = 0x04,
    state = Play,
    direction = Serverbound,
)]
/// Must have at least op level 2 to use. Appears to only be used on singleplayer; the difficulty buttons are still disabled in multiplayer.
///
/// Serverbound `change_difficulty`, Play ID: 4 (0x4)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Difficulty_2)
pub struct ChangeDifficultyServerbound {
    /// 0: peaceful, 1: easy, 2: normal, 3: hard.
    pub difficulty: Difficulty,
}

#[derive(PacketCodec)]
#[packet(
    name = "change_difficulty",
    id = 0x0A,
    state = Play,
    direction = Clientbound,
)]
/// Changes the difficulty setting in the client's option menu
///
/// Clientbound `change_difficulty`, Play ID: 10 (0xA)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Difficulty)
pub struct ChangeDifficultyClientbound {
    /// 0: peaceful, 1: easy, 2: normal, 3: hard.
    pub difficulty: Difficulty,
    /// Difficulty locked?
    pub difficulty_locked: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "block_destruction",
    id = 0x05,
    state = Play,
    direction = Clientbound,
)]
/// 0–9 are the displayable destroy stages and each other number means that there is no animation on this coordinate.
///
/// Block break animations can still be applied on air; the animation will remain visible, although there is no block being broken. However, if this is applied to a transparent block, odd graphical effects may happen, including water losing its transparency. (An effect similar to this can be seen in normal gameplay when breaking ice blocks)
///
/// If you need to display several break animations at the same time, you have to give each of them a unique Entity ID. The entity ID does not need to correspond to an actual entity on the client. It is valid to use a randomly generated number.
///
/// When removing the break animation, you must use the ID of the entity that set it.
///
/// Clientbound `block_destruction`, Play ID: 5 (0x5)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Block_Destroy_Stage)
pub struct SetBlockDestroyStage {
    /// The ID of the entity breaking the block.
    pub entity_id: VarInt,
    /// Block Position.
    pub location: Position,
    /// 0–9 to set it, any other value to remove it.
    pub destroy_stage: UnsignedByte,
}

/// Game mode of the player.
///
/// 0: survival, 1: creative, 2: adventure, 3: spectator.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Game_Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum GameMode {
    /// Survival.
    Survival = 0,
    /// Creative.
    Creative = 1,
    /// Adventure.
    Adventure = 2,
    /// Spectator.
    Spectator = 3,
}

#[derive(PacketCodec)]
#[packet(
    name = "change_game_mode",
    id = 0x05,
    state = Play,
    direction = Serverbound,
)]
/// Requests for the server to update our game mode. Has no effect on vanilla servers if the client doesn't have the required permissions.
///
/// Serverbound `change_game_mode`, Play ID: 5 (0x5)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Game_Mode)
pub struct ChangeGameMode {
    /// 0: survival, 1: creative, 2: adventure, 3: spectator.
    pub game_mode: GameMode,
}

#[derive(PacketCodec)]
#[packet(
    name = "block_entity_data",
    id = 0x06,
    state = Play,
    direction = Clientbound,
)]
/// Sets the block entity associated with the block at the given location.
///
/// Clientbound `block_entity_data`, Play ID: 6 (0x6)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Entity_Data)
pub struct BlockEntityData {
    /// Location
    pub location: Position,
    /// ID in the `minecraft:block_entity_type` registry
    pub r#type: VarInt,
    /// Data to set.
    pub data: Nbt,
}

#[derive(PacketCodec)]
#[packet(
    name = "chat_ack",
    id = 0x06,
    state = Play,
    direction = Serverbound,
)]
/// Serverbound `chat_ack`, Play ID: 6 (0x6)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Message)
pub struct AcknowledgeMessage {
    /// Message Count
    pub message_count: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "chat_command",
    id = 0x07,
    state = Play,
    direction = Serverbound,
)]
/// Main article: [Java Edition protocol/Chat](https://minecraft.wiki/w/Java_Edition_protocol/Chat)
///
/// Serverbound `chat_command`, Play ID: 7 (0x7)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Command)
pub struct ChatCommand {
    /// The command typed by the client excluding the `/`.
    pub command: PrefixedString,
}

/// Action performed by a piston in the Block Action packet.
///
/// 0 to extend the piston, 1 to retract it, and 2 to cancel an ongoing extension.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Block_actions#Piston)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = UnsignedByte)]
pub enum PistonAction {
    /// Extend the piston.
    Extend = 0,
    /// Retract the piston.
    Retract = 1,
    /// Cancel an ongoing extension.
    CancelExtension = 2,
}

/// Direction a piston faces or a bell was rung from.
///
/// down=0, up=1, north=2, south=3, west=4, east=5.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Block_actions#Piston)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = UnsignedByte)]
pub enum Direction {
    /// Down.
    Down = 0,
    /// Up.
    Up = 1,
    /// North.
    North = 2,
    /// South.
    South = 3,
    /// West.
    West = 4,
    /// East.
    East = 5,
}

#[derive(PacketCodec)]
#[packet(
    name = "block_event",
    id = 0x07,
    state = Play,
    direction = Clientbound,
)]
/// This packet is used for a number of actions and animations performed by blocks, usually non-persistent. The client ignores the provided block type and instead uses the block state in their world.
///
/// See [Java Edition protocol/Block actions](https://minecraft.wiki/w/Java_Edition_protocol/Block_actions) for a list of values.
///
/// This packet uses a block ID from the `minecraft:block` registry, not a block state.
///
/// Clientbound `block_event`, Play ID: 7 (0x7)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Action)
pub struct BlockAction {
    /// Block coordinates.
    pub location: Position,
    /// Varies depending on block — see [Java Edition protocol/Block actions](https://minecraft.wiki/w/Java_Edition_protocol/Block_actions).
    pub action_id: UnsignedByte,
    /// Varies depending on block — see [Java Edition protocol/Block actions](https://minecraft.wiki/w/Java_Edition_protocol/Block_actions).
    pub action_parameter: UnsignedByte,
    /// ID in the `minecraft:block` registry. This value is unused by the vanilla client, as it will infer the type of block based on the given position.
    pub block_type: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "block_update",
    id = 0x08,
    state = Play,
    direction = Clientbound,
)]
/// Fired whenever a block is changed within the render distance.
///
/// Clientbound `block_update`, Play ID: 8 (0x8)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Update)
pub struct BlockUpdate {
    /// Block Coordinates.
    pub location: Position,
    /// The new block state ID for the block as given in the [global block state palette](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Global_block_state_palette).
    pub block_id: VarInt,
}

/// One signed argument inside the Signed Chat Command packet.
#[derive(Debug, Clone, PartialEq, Eq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct ArgumentSignature {
    /// The name of the argument that is signed by the following signature.
    pub name: BoundedString<16>,
    /// The signature that verifies the argument. Always 256 bytes and is not length-prefixed.
    pub signature: FixedByteArray<256>,
}

#[derive(PacketCodec)]
#[packet(
    name = "chat_command_signed",
    id = 0x08,
    state = Play,
    direction = Serverbound,
)]
/// Main article: [Java Edition protocol/Chat](https://minecraft.wiki/w/Java_Edition_protocol/Chat)
///
/// Serverbound `chat_command_signed`, Play ID: 8 (0x8)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Signed_Chat_Command)
pub struct SignedChatCommand {
    /// The command typed by the client excluding the `/`.
    pub command: PrefixedString,
    /// The timestamp that the command was executed.
    pub timestamp: Long,
    /// The salt for the following argument signatures.
    pub salt: Long,
    /// Array of argument signatures.
    pub signatures: BoundedPrefixedArray<ArgumentSignature, 8>,
    /// Message Count
    pub message_count: VarInt,
    /// Acknowledged
    pub acknowledged: FixedBitSet<20>,
    /// Checksum
    pub checksum: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "chat",
    id = 0x09,
    state = Play,
    direction = Serverbound,
)]
/// Main article: [Java Edition protocol/Chat](https://minecraft.wiki/w/Java_Edition_protocol/Chat)
///
/// Used to send a chat message to the server. The message may not be longer than 256 characters or else the server will kick the client.
///
/// The server will broadcast a [Player Chat Message](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Chat_Message) packet with Chat Type `minecraft:chat` to all players that haven't disabled chat (including the player that sent the message). See [Java Edition protocol/Chat#Processing chat](https://minecraft.wiki/w/Java_Edition_protocol/Chat#Processing_chat) for more information.
///
/// Serverbound `chat`, Play ID: 9 (0x9)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Message)
pub struct ChatMessage {
    /// Content of the message
    pub message: BoundedString<256>,
    /// Number of milliseconds since the epoch (1 Jan 1970, midnight, UTC)
    pub timestamp: Long,
    /// The salt used to verify the signature hash. Randomly generated by the client
    pub salt: Long,
    /// The signature used to verify the chat message's authentication. When present, always 256 bytes and not length-prefixed.
    ///
    /// This is a SHA256 with RSA digital signature computed over the following:
    ///
    /// - The number 1 as a 4-byte int. Always 00 00 00 01.
    /// - The player's 16-byte UUID.
    /// - The chat session (a 16-byte UUID randomly generated by the client).
    /// - The index of the message within this chat session as a 4-byte int. First message is 0, next message is 1, etc. Incremented each time the client sends a chat message.
    /// - The salt (from above) as an 8-byte long.
    /// - The timestamp (from above) converted from milliseconds to seconds, so divide by 1000, as an 8-byte long.
    /// - The length of the message in bytes (from above) as a 4-byte int.
    /// - The message bytes.
    /// - The number of messages in the last seen set, as a 4-byte int. Always in the range [0,20].
    /// - For each message in the last seen set, from oldest to newest, the 256-byte signature of that message.
    ///
    /// The client's chat private key is used for the message signature.
    pub signature: PrefixedOptional<FixedByteArray<256>>,
    /// Number of signed clientbound chat messages the client has seen from the server since the last serverbound chat message from this client. The server will use this to update its last seen list for the client.
    pub message_count: VarInt,
    /// Bitmask of which message signatures from the last seen set were used to sign this message. The most recent is the highest bit. If there are fewer than 20 messages in the last seen set, the lower bits will be zeros.
    pub acknowledged: FixedBitSet<20>,
    /// Checksum is computed over all the message signature checksums in the last seen set, from oldest to newest. Both the packet checksum and signature checksums use the same logic as Java's Arrays.hashCode(byte[]) implementation. The packet checksum additionally casts the resulting int to a byte, and if that byte is 0 returns 1, otherwise returns said byte
    pub checksum: Byte,
}

/// A player chat session's public key.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct PlayerSessionKey {
    /// The time the play session key expires in [epoch](https://en.wikipedia.org/wiki/Unix_time) milliseconds.
    pub expires_at: Long,
    /// A byte array of an X.509-encoded public key. Get this from [Mojang API#Get keypair for signature](https://minecraft.wiki/w/Mojang_API#Get_keypair_for_signature)
    pub public_key: BoundedPrefixedArray<Byte, 512>,
    /// The signature consists of the player UUID, the key expiration timestamp, and the public key data. These values are hashed using [SHA-1](https://en.wikipedia.org/wiki/SHA-1) and signed using Mojang's private [RSA](https://en.wikipedia.org/wiki/RSA_(cryptosystem)) key.
    pub key_signature: BoundedPrefixedArray<Byte, 4096>,
}

#[derive(PacketCodec)]
#[packet(
    name = "chat_session_update",
    id = 0x0A,
    state = Play,
    direction = Serverbound,
)]
/// Serverbound `chat_session_update`, Play ID: 10 (0xA)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Session)
pub struct PlayerSession {
    /// Session Id
    pub session_id: Uuid,
    /// Public Key
    pub public_key: PlayerSessionKey,
}
