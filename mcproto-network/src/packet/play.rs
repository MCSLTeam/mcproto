//! Play state packets for protocol 777 (Minecraft Java Edition 26.3).

pub mod bossbar;
pub mod chunk;
pub mod container;
pub mod debug;

pub use bossbar::*;
pub use chunk::*;
pub use container::*;
pub use debug::*;

use mcproto_codec::error::{CodecError, CodecKind};
use mcproto_types::{
    Angle, Boolean, BoundedPrefixedArray, BoundedString, Byte, CommandNode, Context,
    ContextualCodec, Double, FixedBitSet, FixedByteArray, Float, Identifier, Int, Long, LpVec3,
    Nbt, Optional, Position, PrefixedArray, PrefixedOptional, PrefixedString, ProtocolEnum,
    RemainingBytes, Short, Slot, TextComponent, TypeCodec, TypeStructCodec, UnsignedByte, Uuid,
    VarInt, VarLong,
};

use crate::packet::configuration::{
    ChatMode, ClientLocale, CustomClickPayload, DisplayedSkinParts, MainHand, ParticleStatus,
    ResourcePackResult,
};
use crate::{EncodePacket, Packet, PacketCodec, PacketId, PacketName, ProtocolState};

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

/// Action performed by the Client Status packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Status)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ClientStatusAction {
    /// Sent when the client is ready to respawn after death.
    PerformRespawn = 0,
    /// Sent when the client opens the Statistics menu.
    RequestStats = 1,
    /// Sent when the client opens the Edit Game Rules menu.
    RequestGameRuleValues = 2,
}

#[derive(PacketCodec)]
#[packet(
    name = "client_command",
    id = 0x0C,
    state = Play,
    direction = Serverbound,
)]
/// Serverbound `client_command`, Play ID: 12 (0xC)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Status)
pub struct ClientStatus {
    /// Action ID.
    pub action: ClientStatusAction,
}

#[derive(PacketCodec)]
#[packet(
    name = "clear_titles",
    id = 0x0E,
    state = Play,
    direction = Clientbound,
)]
/// Clear the client's current title information, with the option to also
/// reset it.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Titles)
pub struct ClearTitles {
    /// Reset
    pub reset: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "client_tick_end",
    id = 0x0D,
    state = Play,
    direction = Serverbound,
)]
/// Serverbound `client_tick_end`, Play ID: 13 (0xD)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Tick_End)
pub struct ClientTickEnd;

#[derive(PacketCodec)]
#[packet(
    name = "client_information",
    id = 0x0E,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the player connects, or when settings are changed.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information)
pub struct ClientInformation {
    /// e.g. `en_GB`.
    pub locale: ClientLocale,
    /// Client-side render distance, in chunks.
    pub view_distance: Byte,
    /// Which chat messages the client displays.
    pub chat_mode: ChatMode,
    /// Whether the multiplayer Colors setting is enabled.
    pub chat_colors: Boolean,
    /// Enabled skin layers.
    pub displayed_skin_parts: DisplayedSkinParts,
    /// Player's selected main hand.
    pub main_hand: MainHand,
    /// Whether account text filtering is enabled.
    pub enable_text_filtering: Boolean,
    /// Whether the player permits inclusion in server player listings.
    pub allow_server_listings: Boolean,
    /// Client particle-density preference.
    pub particle_status: ParticleStatus,
}

/// One completion candidate inside the Command Suggestions Response packet.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct CommandSuggestionMatch {
    /// One eligible value to insert, note that each command is sent separately
    /// instead of in a single string, hence the need for Count.
    pub r#match: PrefixedString,
    /// Tooltip to display.
    pub tooltip: PrefixedOptional<TextComponent>,
}

#[derive(PacketCodec)]
#[packet(
    name = "command_suggestion",
    id = 0x0F,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the client needs to tab-complete a `minecraft:ask_server`
/// suggestion type.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Command_Suggestions_Request)
pub struct CommandSuggestionsRequest {
    /// The ID of the transaction that the server will send back to the client
    /// in the response of this packet. Client generates this and increments it
    /// each time it sends another tab completion that doesn't get a response.
    pub transaction_id: VarInt,
    /// All the text behind the cursor including the `/` (e.g. to the left of
    /// the cursor in left-to-right languages like English).
    pub text: BoundedString<32500>,
}

#[derive(PacketCodec)]
#[packet(
    name = "command_suggestions",
    id = 0x0F,
    state = Play,
    direction = Clientbound,
)]
/// The server responds with a list of auto-completions of the last word sent
/// to it. In the case of regular chat, this is a player username. Command
/// names and parameters are also supported. The client sorts these
/// alphabetically before listing them.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Command_Suggestions_Response)
pub struct CommandSuggestionsResponse {
    /// Transaction ID.
    pub id: VarInt,
    /// Start of the text to replace.
    pub start: VarInt,
    /// Length of the text to replace.
    pub length: VarInt,
    /// Completion candidates.
    pub matches: PrefixedArray<CommandSuggestionMatch>,
}

#[derive(PacketCodec)]
#[packet(
    name = "commands",
    id = 0x10,
    state = Play,
    direction = Clientbound,
)]
/// Lists all of the commands on the server, and how they are parsed.
///
/// This is a directed graph, with one root node. Each redirect or child node
/// must refer only to nodes that have already been declared.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Commands)
pub struct Commands {
    /// An array of nodes.
    pub nodes: PrefixedArray<CommandNode>,
    /// Index of the `root` node in the previous array.
    pub root_index: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "configuration_acknowledged",
    id = 0x10,
    state = Play,
    direction = Serverbound,
)]
/// Sent by the client upon receiving a Start Configuration packet from the
/// server.
///
/// This packet switches the connection state to configuration.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Configuration)
pub struct AcknowledgeConfiguration;
#[derive(PacketCodec)]
#[packet(
    name = "cookie_request",
    id = 0x15,
    state = Play,
    direction = Clientbound,
)]
/// Requests a cookie that was previously stored.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request)
pub struct CookieRequest {
    /// The identifier of the cookie.
    pub key: Identifier,
}

#[derive(PacketCodec)]
#[packet(
    name = "cookie_response",
    id = 0x15,
    state = Play,
    direction = Serverbound,
)]
/// Response to a [Cookie Request](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request) from the server.
///
/// The vanilla server only accepts responses of up to 5 kiB in size.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response)
pub struct CookieResponse {
    /// The identifier of the cookie.
    pub key: Identifier,
    /// The data of the cookie.
    pub payload: PrefixedOptional<BoundedPrefixedArray<Byte, 5120>>,
}
/// Action performed by the Chat Suggestions packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Suggestions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ChatSuggestionsAction {
    /// Add entries.
    Add = 0,
    /// Remove entries.
    Remove = 1,
    /// Set entries.
    Set = 2,
}

#[derive(PacketCodec)]
#[packet(
    name = "cooldown",
    id = 0x16,
    state = Play,
    direction = Clientbound,
)]
/// Applies a cooldown period to all items with the given type. Used by the
/// vanilla server with enderpearls. This packet should be sent when the
/// cooldown starts and also when the cooldown ends (to compensate for lag),
/// although the client will end the cooldown automatically. Can be applied to
/// any item, note that interactions still get sent to the server with the item,
/// but the client does not play the animation nor attempt to predict results
/// (i.e, block placing).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Cooldown)
pub struct SetCooldown {
    /// Identifier of the item (`minecraft:stone`) or the cooldown group
    /// (`use_cooldown` item component).
    pub cooldown_group: Identifier,
    /// Number of ticks to apply a cooldown for, or 0 to clear the cooldown.
    pub cooldown_ticks: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "custom_chat_completions",
    id = 0x17,
    state = Play,
    direction = Clientbound,
)]
/// Unused by the vanilla server. Likely provided for custom servers to send
/// chat message completions to clients.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Suggestions)
pub struct ChatSuggestions {
    /// 0: Add, 1: Remove, 2: Set.
    pub action: ChatSuggestionsAction,
    /// Chat suggestion entries.
    pub entries: PrefixedArray<BoundedString<32767>>,
}

#[derive(PacketCodec)]
#[packet(
    name = "custom_payload",
    id = 0x18,
    state = Play,
    direction = Clientbound,
)]
/// Mods and plugins can use this to send their data. Minecraft itself uses
/// several plugin channels. These internal channels are in the `minecraft`
/// namespace.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound))
pub struct PluginMessageClientbound {
    /// Name of the plugin channel used to send the data.
    pub channel: Identifier,
    /// Any data, depending on the channel. The payload occupies the rest of
    /// the packet body without a universal length prefix.
    pub data: RemainingBytes,
}

#[derive(PacketCodec)]
#[packet(
    name = "custom_payload",
    id = 0x16,
    state = Play,
    direction = Serverbound,
)]
/// Mods and plugins can use this to send their data. Minecraft itself uses
/// some plugin channels. These internal channels are in the `minecraft`
/// namespace.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound))
pub struct PluginMessageServerbound {
    /// Name of the plugin channel used to send the data.
    pub channel: Identifier,
    /// Any data, depending on the channel. The payload occupies the rest of
    /// the packet body without a universal length prefix.
    pub data: RemainingBytes,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_subscription_request",
    id = 0x17,
    state = Play,
    direction = Serverbound,
)]
/// Sent by the client whenever debug subscriptions, used by debug graphs and
/// renderers, are activated or deactivated. The list in the packet replaces
/// the previous set of active subscriptions. Subscriptions not in the list are
/// deactivated.
///
/// If the client does not have permission to receive the requested debug
/// information, the subscriptions are nonetheless retained by the server, and
/// it is not necessary to send this packet again if the permissions change.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Subscription_Request)
pub struct DebugSubscriptionRequest {
    /// List of active debug subscriptions. IDs in the
    /// `minecraft:debug_subscription` registry.
    pub subscriptions: PrefixedArray<VarInt>,
}

#[derive(PacketCodec)]
#[packet(
    name = "damage_event",
    id = 0x19,
    state = Play,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Damage_Event)
pub struct DamageEvent {
    /// The ID of the entity taking damage.
    pub entity_id: VarInt,
    /// The type of damage in the `minecraft:damage_type` registry, defined by
    /// the Registry Data packet.
    pub source_type_id: VarInt,
    /// The ID + 1 of the entity responsible for the damage, if present. If not
    /// present, the value is 0.
    pub source_cause_id: VarInt,
    /// The ID + 1 of the entity that directly dealt the damage, if present. If
    /// not present, the value is 0. If this field is present and damage was
    /// dealt indirectly, such as by the use of a projectile, this field will
    /// contain the ID of such projectile; and if damage was dealt directly,
    /// such as by manually attacking, this field will contain the same value as
    /// Source Cause ID.
    pub source_direct_id: VarInt,
    /// The vanilla server sends the Source Position when the damage was dealt
    /// by the /damage command and a position was specified.
    pub source_position: PrefixedOptional<DamageEventSourcePosition>,
}

/// Source position for the Damage Event packet.
#[derive(Debug, Clone, Copy, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct DamageEventSourcePosition {
    /// X coordinate.
    pub x: Double,
    /// Y coordinate.
    pub y: Double,
    /// Z coordinate.
    pub z: Double,
}

#[derive(PacketCodec)]
#[packet(
    name = "edit_book",
    id = 0x18,
    state = Play,
    direction = Serverbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Edit_Book)
pub struct EditBook {
    /// The hotbar slot where the written book is located.
    pub slot: VarInt,
    /// Text from each page. Maximum string length is 1024 chars.
    pub entries: BoundedPrefixedArray<BoundedString<1024>, 100>,
    /// Title of book. Present if book is being signed, not present if book is
    /// being edited.
    pub title: PrefixedOptional<BoundedString<32>>,
}

#[derive(PacketCodec)]
#[packet(
    name = "entity_tag_query",
    id = 0x19,
    state = Play,
    direction = Serverbound,
)]
/// Used when F3+I is pressed while looking at an entity.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Query_Entity_Tag)
pub struct QueryEntityTag {
    /// An incremental ID so that the client can verify that the response
    /// matches.
    pub transaction_id: VarInt,
    /// The ID of the entity to query.
    pub entity_id: VarInt,
}
/// Hand used for an interaction.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Interact)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum InteractionHand {
    /// Main hand.
    MainHand = 0,
    /// Off hand.
    OffHand = 1,
}

#[derive(PacketCodec)]
#[packet(
    name = "interact",
    id = 0x1A,
    state = Play,
    direction = Serverbound,
)]
/// This packet is sent from the client to the server when the client right-clicks another entity (a player, minecart, etc).
///
/// A vanilla server only accepts this packet if the entity being clicked is visible without obstruction and within a 4-unit radius of the player's position.
///
/// The target offset field represents the difference between the vector location of the cursor at the time of the packet and the entity's position.
///
/// Note that middle-click in creative mode is interpreted by the client and sent as a [Set Creative Mode Slot](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Creative_Mode_Slot) packet instead.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Interact)
pub struct Interact {
    /// The ID of the entity to interact. Note the special case described below.
    pub entity_id: VarInt,
    /// 0: main hand, 1: off hand.
    pub hand: InteractionHand,
    /// The difference between the vector location of the cursor at the time of the packet and the entity's position.
    pub target_offset: LpVec3,
    /// If the client is pressing the sneak key. Has the same effect as a Player Command Press/Release sneak key preceding the interaction, and the state is permanently changed.
    pub sneak_key_pressed: Boolean,
}
#[derive(PacketCodec)]
#[packet(
    name = "jigsaw_generate",
    id = 0x1B,
    state = Play,
    direction = Serverbound,
)]
/// Sent when Generate is pressed on the [Jigsaw Block](https://minecraft.wiki/w/Jigsaw_Block) interface.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Jigsaw_Generate)
pub struct JigsawGenerate {
    /// Block entity location.
    pub location: Position,
    /// Value of the levels slider/max depth to generate.
    pub levels: VarInt,
    /// Keep Jigsaws.
    pub keep_jigsaws: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "keep_alive",
    id = 0x1C,
    state = Play,
    direction = Serverbound,
)]
/// The server will frequently send out a keep-alive (see [Keep Alive (clientbound)](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound))), each containing a random ID. The client must respond with the same packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound))
pub struct KeepAliveServerbound {
    /// Keep Alive ID.
    pub keep_alive_id: Long,
}

#[derive(PacketCodec)]
#[packet(
    name = "lock_difficulty",
    id = 0x1D,
    state = Play,
    direction = Serverbound,
)]
/// Must have at least op level 2 to use. Appears to only be used on singleplayer; the difficulty buttons are still disabled in multiplayer.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Lock_Difficulty)
pub struct LockDifficulty {
    /// Locked.
    pub locked: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "move_player_pos",
    id = 0x1E,
    state = Play,
    direction = Serverbound,
)]
/// Updates the player's XYZ position on the server.
///
/// If the player is in a vehicle, the position is ignored. If the player is
/// sleeping, the position is not changed, and a Synchronize Player Position is
/// sent if the received position deviated from the server's view by more than a
/// meter.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Position)
pub struct SetPlayerPosition {
    /// Absolute position.
    pub x: Double,
    /// Absolute feet position, normally Head Y - 1.62.
    pub feet_y: Double,
    /// Absolute position.
    pub z: Double,
    /// Bit field: 0x01: on ground, 0x02: pushing against wall.
    pub flags: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "move_player_pos_rot",
    id = 0x1F,
    state = Play,
    direction = Serverbound,
)]
/// A combination of Set Player Rotation and Set Player Position.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Position_and_Rotation)
pub struct SetPlayerPositionAndRotation {
    /// Absolute position.
    pub x: Double,
    /// Absolute feet position, normally Head Y - 1.62.
    pub feet_y: Double,
    /// Absolute position.
    pub z: Double,
    /// Absolute rotation on the X Axis, in degrees.
    pub yaw: Float,
    /// Absolute rotation on the Y Axis, in degrees.
    pub pitch: Float,
    /// Bit field: 0x01: on ground, 0x02: pushing against wall.
    pub flags: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "move_player_rot",
    id = 0x20,
    state = Play,
    direction = Serverbound,
)]
/// Updates the direction the player is looking in.
///
/// Yaw is measured in degrees and does not follow classical trigonometry
/// rules. The unit circle of yaw on the XZ-plane starts at (0, 1) and turns
/// counterclockwise, with 90 at (-1, 0), 180 at (0,-1) and 270 at (1, 0).
/// Additionally, yaw is not clamped to between 0 and 360 degrees; any number
/// is valid, including negative numbers and numbers greater than 360.
///
/// Pitch is measured in degrees, where 0 is looking straight ahead, -90 is
/// looking straight up, and 90 is looking straight down.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Rotation)
pub struct SetPlayerRotation {
    /// Absolute rotation on the X Axis, in degrees.
    pub yaw: Float,
    /// Absolute rotation on the Y Axis, in degrees.
    pub pitch: Float,
    /// Bit field: 0x01: on ground, 0x02: pushing against wall.
    pub flags: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "move_player_status_only",
    id = 0x21,
    state = Play,
    direction = Serverbound,
)]
/// This packet is used to indicate whether the player is on ground
/// (walking/swimming) or airborne (jumping/falling).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Movement_Flags)
pub struct SetPlayerMovementFlags {
    /// Bit field: 0x01: on ground, 0x02: pushing against wall.
    pub flags: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "move_vehicle",
    id = 0x22,
    state = Play,
    direction = Serverbound,
)]
/// Sent when a player moves in a client-side-controlled vehicle. Fields are
/// the same position and rotation values as in Set Player Position and
/// Rotation, followed by On Ground. All position and rotation fields are
/// absolute.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Move_Vehicle_(serverbound))
pub struct MoveVehicleServerbound {
    /// Absolute position (X coordinate).
    pub x: Double,
    /// Absolute position (Y coordinate).
    pub y: Double,
    /// Absolute position (Z coordinate).
    pub z: Double,
    /// Absolute rotation on the vertical axis, in degrees.
    pub yaw: Float,
    /// Absolute rotation on the horizontal axis, in degrees.
    pub pitch: Float,
    /// Whether the vehicle is on the ground.
    pub on_ground: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "paddle_boat",
    id = 0x23,
    state = Play,
    direction = Serverbound,
)]
/// Used to *visually* update whether boat paddles are turning. The server
/// will update the Boat entity metadata to match the values here.
///
/// Right paddle turning is set to true when the left button or forward button
/// is held, left paddle turning is set to true when the right button or
/// forward button is held.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Paddle_Boat)
pub struct PaddleBoat {
    /// Left paddle turning.
    pub left_paddle_turning: Boolean,
    /// Right paddle turning.
    pub right_paddle_turning: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "pick_item_from_block",
    id = 0x24,
    state = Play,
    direction = Serverbound,
)]
/// Used for pick block functionality (middle click) on blocks to retrieve
/// items from the inventory in survival or creative mode or create them in
/// creative mode.
///
/// See [Controls#Pick Block](https://minecraft.wiki/w/Controls#Pick_Block) for
/// more information.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pick_Item_From_Block)
pub struct PickItemFromBlock {
    /// The location of the block.
    pub location: Position,
    /// Used to tell the server to include block data in the new stack, works
    /// only if in creative mode.
    pub include_data: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "pick_item_from_entity",
    id = 0x25,
    state = Play,
    direction = Serverbound,
)]
/// Used for pick block functionality (middle click) on entities to retrieve
/// items from the inventory in survival or creative mode or create them in
/// creative mode.
///
/// See [Controls#Pick Block](https://minecraft.wiki/w/Controls#Pick_Block) for
/// more information.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pick_Item_From_Entity)
pub struct PickItemFromEntity {
    /// The ID of the entity to pick.
    pub entity_id: VarInt,
    /// Unused by the vanilla server.
    pub include_data: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "ping_request",
    id = 0x26,
    state = Play,
    direction = Serverbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Request)
pub struct PingRequest {
    /// May be any number, but vanilla clients will always use the timestamp in
    /// milliseconds.
    pub id: Long,
}

#[derive(PacketCodec)]
#[packet(
    name = "place_recipe",
    id = 0x27,
    state = Play,
    direction = Serverbound,
)]
/// This packet is sent when a player clicks a recipe in the crafting book that
/// is craftable (white border).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Place_Recipe)
pub struct PlaceRecipe {
    /// Window ID.
    pub window_id: VarInt,
    /// ID of recipe previously defined in Recipe Book Add.
    pub recipe_id: VarInt,
    /// Affects the amount of items processed; true if shift is down when
    /// clicked.
    pub make_all: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "player_abilities",
    id = 0x28,
    state = Play,
    direction = Serverbound,
)]
/// The vanilla client sends this packet when the player starts/stops flying
/// with the Flags parameter changed accordingly.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Abilities_(serverbound))
pub struct PlayerAbilitiesServerbound {
    /// Bit mask. 0x02: is flying.
    pub flags: Byte,
}

/// The action the player is taking against the block.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum PlayerActionStatus {
    /// Started digging.
    StartedDigging = 0,
    /// Cancelled digging.
    CancelledDigging = 1,
    /// Finished digging.
    FinishedDigging = 2,
    /// Drop item stack.
    DropItemStack = 3,
    /// Drop item.
    DropItem = 4,
    /// Shoot arrow / finish eating.
    ShootArrowOrFinishEating = 5,
    /// Swap item in hand.
    SwapItemInHand = 6,
    /// Stab.
    Stab = 7,
}

/// The face being hit.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = Byte)]
pub enum BlockFace {
    /// Bottom.
    Bottom = 0,
    /// Top.
    Top = 1,
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
    name = "player_action",
    id = 0x29,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the player mines a block.
///
/// A vanilla server only accepts digging packets with coordinates within a
/// 6-unit radius between the center of the block and the player's eyes.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action)
pub struct PlayerAction {
    /// The action the player is taking against the block.
    pub status: PlayerActionStatus,
    /// Block position.
    pub location: Position,
    /// The face being hit.
    pub face: BlockFace,
    /// Block change sequence number (see Acknowledge Block Change).
    pub sequence: VarInt,
}

/// The action performed by the Player Command packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Command)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum PlayerCommandAction {
    /// Leave bed.
    LeaveBed = 0,
    /// Start sprinting.
    StartSprinting = 1,
    /// Stop sprinting.
    StopSprinting = 2,
    /// Start jump with horse.
    StartJumpWithHorse = 3,
    /// Stop jump with horse.
    StopJumpWithHorse = 4,
    /// Open vehicle inventory.
    OpenVehicleInventory = 5,
    /// Start flying with elytra.
    StartFlyingWithElytra = 6,
}

#[derive(PacketCodec)]
#[packet(
    name = "player_command",
    id = 0x2A,
    state = Play,
    direction = Serverbound,
)]
/// Sent by the client to indicate that it has performed certain actions:
/// sprinting, exiting a bed, jumping with a horse, and opening a horse's
/// inventory while riding it.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Command)
pub struct PlayerCommand {
    /// Player ID (ignored by the vanilla server).
    pub entity_id: VarInt,
    /// The ID of the action.
    pub action_id: PlayerCommandAction,
    /// Only used by the "start jump with horse" action, in which case it ranges
    /// from 0 to 100. In all other cases it is 0.
    pub jump_boost: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "player_input",
    id = 0x2B,
    state = Play,
    direction = Serverbound,
)]
/// Sent whenever the player presses or releases certain keys. The flags
/// correspond directly to the states of their corresponding keys; the Sprint
/// flag does not depend on whether the player is actually able to sprint at
/// the moment.
///
/// Used by the vanilla server for minecart controls, player inputs in the
/// `entity_properties` predicate, and sneaking (sprinting is still controlled
/// by Player Command).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Input)
pub struct PlayerInput {
    /// Bit mask:
    ///
    /// - 0x01: Forward
    /// - 0x02: Backward
    /// - 0x04: Left
    /// - 0x08: Right
    /// - 0x10: Jump
    /// - 0x20: Sneak
    /// - 0x40: Sprint
    pub flags: UnsignedByte,
}

#[derive(PacketCodec)]
#[packet(
    name = "player_loaded",
    id = 0x2C,
    state = Play,
    direction = Serverbound,
)]
/// Sent by the client to indicate that it is ready to start simulating the
/// player. The vanilla client sends this when the "Loading terrain..." screen
/// is closed.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Loaded)
pub struct PlayerLoaded;

#[derive(PacketCodec)]
#[packet(
    name = "pong",
    id = 0x2D,
    state = Play,
    direction = Serverbound,
)]
/// Response to the clientbound Ping packet with the same ID.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong)
pub struct Pong {
    /// Ping ID copied from the request.
    pub id: Int,
}

#[derive(PacketCodec)]
#[packet(
    name = "punch",
    id = 0x2E,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the client punches.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Punch)
pub struct Punch;

/// Recipe book selected by the Change Recipe Book Settings packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Recipe_Book_Settings)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum RecipeBookType {
    /// Crafting recipe book.
    Crafting = 0,
    /// Furnace recipe book.
    Furnace = 1,
    /// Blast furnace recipe book.
    BlastFurnace = 2,
    /// Smoker recipe book.
    Smoker = 3,
}

#[derive(PacketCodec)]
#[packet(
    name = "recipe_book_change_settings",
    id = 0x2F,
    state = Play,
    direction = Serverbound,
)]
/// Replaces Recipe Book Data, type 1.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Recipe_Book_Settings)
pub struct ChangeRecipeBookSettings {
    /// Recipe book being changed.
    pub book_id: RecipeBookType,
    /// Whether the recipe book is open.
    pub book_open: Boolean,
    /// Whether the recipe book filter is active.
    pub filter_active: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "recipe_book_seen_recipe",
    id = 0x30,
    state = Play,
    direction = Serverbound,
)]
/// Sent when a recipe is first seen in the recipe book. Replaces Recipe Book
/// Data, type 0.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Seen_Recipe)
pub struct SetSeenRecipe {
    /// ID of a recipe previously defined in Recipe Book Add.
    pub recipe_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "rename_item",
    id = 0x31,
    state = Play,
    direction = Serverbound,
)]
/// Sent as a player is renaming an item in an anvil; each keypress in the
/// anvil UI sends a new Rename Item packet. If the new name is empty, then the
/// item loses its custom name.
///
/// The item name may be no longer than 50 characters, and if it is longer than
/// that, then the rename is silently ignored.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Rename_Item)
pub struct RenameItem {
    /// The new name of the item.
    pub item_name: BoundedString<32767>,
}

#[derive(PacketCodec)]
#[packet(
    name = "resource_pack",
    id = 0x32,
    state = Play,
    direction = Serverbound,
)]
/// Reports the client's handling of a resource-pack request.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response)
pub struct ResourcePackResponse {
    /// The unique identifier of the resource pack received in the Add Resource
    /// Pack request.
    pub uuid: Uuid,
    /// Current result of processing the resource pack.
    pub result: ResourcePackResult,
}

/// Action performed by the Seen Advancements packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Seen_Advancements)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SeenAdvancementsAction {
    /// Opened tab.
    OpenedTab = 0,
    /// Closed screen.
    ClosedScreen = 1,
}

/// Informs the server which advancements tab is open or that the screen closed.
///
/// The tab identifier is present only when the action is
/// [`OpenedTab`](SeenAdvancementsAction::OpenedTab).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Seen_Advancements)
#[derive(Debug, Clone, PartialEq)]
pub struct SeenAdvancements {
    /// The advancement-tab action being reported.
    pub action: SeenAdvancementsAction,
    /// Advancement tab identifier, present only when a tab was opened.
    pub tab_id: Optional<Identifier>,
}

impl Packet for SeenAdvancements {
    const NAME: PacketName = PacketName::new("seen_advancements");
    const ID: PacketId = match PacketId::new(0x33) {
        Some(id) => id,
        None => panic!("packet ID must be non-negative"),
    };
    const STATE: ProtocolState = ProtocolState::Play;
    const DIRECTION: crate::Direction = crate::Direction::Serverbound;
}

impl EncodePacket for SeenAdvancements {
    fn encode_body(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        self.action
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::TypeStruct))?;
        self.tab_id
            .encode_with_context(
                writer,
                &Context::new(self.action == SeenAdvancementsAction::OpenedTab),
            )
            .map_err(|error| error.with_context(CodecKind::TypeStruct))
    }
}

#[derive(PacketCodec)]
#[packet(
    name = "select_trade",
    id = 0x34,
    state = Play,
    direction = Serverbound,
)]
/// Sent when a player selects a specific trade offered by a villager NPC.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Select_Trade)
pub struct SelectTrade {
    /// The selected slot in the player's current (trading) inventory.
    pub selected_slot: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_beacon",
    id = 0x35,
    state = Play,
    direction = Serverbound,
)]
/// Changes the effect of the current beacon.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Beacon_Effect)
pub struct SetBeaconEffect {
    /// A Potion ID.
    pub primary_effect: PrefixedOptional<VarInt>,
    /// A Potion ID.
    pub secondary_effect: PrefixedOptional<VarInt>,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_carried_item",
    id = 0x36,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the player changes the slot selection.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Held_Item_(serverbound))
pub struct SetHeldItemServerbound {
    /// The slot which the player has selected (0-8).
    pub slot: Short,
}

/// Mode of a command block programmed by the client.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum CommandBlockMode {
    /// Chain command block.
    Chain = 0,
    /// Repeating command block.
    Repeating = 1,
    /// Impulse command block.
    Impulse = 2,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_command_block",
    id = 0x37,
    state = Play,
    direction = Serverbound,
)]
/// Programs a command block at the supplied location.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block)
pub struct ProgramCommandBlock {
    /// Command-block location.
    pub location: Position,
    /// Command to store in the block.
    pub command: BoundedString<32767>,
    /// Command-block execution mode.
    pub mode: CommandBlockMode,
    /// 0x01: Track Output (if false, the output of the previous command will
    /// not be stored within the command block); 0x02: Is conditional; 0x04:
    /// Automatic.
    pub flags: Byte,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_command_minecart",
    id = 0x38,
    state = Play,
    direction = Serverbound,
)]
/// Programs a command-block minecart.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block_Minecart)
pub struct ProgramCommandBlockMinecart {
    /// Runtime entity ID of the command-block minecart.
    pub entity_id: VarInt,
    /// Command to store in the minecart.
    pub command: BoundedString<32767>,
    /// If false, the output of the previous command will not be stored within
    /// the command block.
    pub track_output: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_creative_mode_slot",
    id = 0x39,
    state = Play,
    direction = Serverbound,
)]
/// Sets a slot while the user is in the standard inventory in Creative mode.
///
/// Clicking in the creative inventory menu is quite different from
/// non-creative inventory management. Picking up an item with the mouse
/// deletes the item from the server, and placing an item into a slot or
/// dropping it out of the inventory tells the server to create the item.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Creative_Mode_Slot)
pub struct SetCreativeModeSlot {
    /// Inventory slot.
    pub slot: Short,
    /// Item to place in the slot.
    pub clicked_item: Slot,
}

/// One game-rule assignment inside the Set Game Rules packet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct GameRule {
    /// Game-rule identifier.
    pub name: Identifier,
    /// New game-rule value.
    pub value: BoundedString<32767>,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_game_rule",
    id = 0x3A,
    state = Play,
    direction = Serverbound,
)]
/// Changes one or more game rules.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Game_Rules)
pub struct SetGameRules {
    /// Game-rule names and their new values.
    pub rules: PrefixedArray<GameRule>,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_jigsaw_block",
    id = 0x3B,
    state = Play,
    direction = Serverbound,
)]
/// Sent when Done is pressed on the Jigsaw Block interface.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Jigsaw_Block)
pub struct ProgramJigsawBlock {
    /// Block entity location.
    pub location: Position,
    /// Jigsaw block name.
    pub name: Identifier,
    /// Target jigsaw pool element.
    pub target: Identifier,
    /// Jigsaw pool.
    pub pool: Identifier,
    /// "Turns into" on the GUI, `final_state` in NBT.
    pub final_state: BoundedString<32767>,
    /// `rollable` if the attached piece can be rotated, else `aligned`.
    pub joint_type: BoundedString<32767>,
    /// Selection priority.
    pub selection_priority: VarInt,
    /// Placement priority.
    pub placement_priority: VarInt,
}

/// Additional action performed by the Program Structure Block packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum StructureBlockAction {
    /// Update data.
    UpdateData = 0,
    /// Save the structure.
    Save = 1,
    /// Load the structure.
    Load = 2,
    /// Detect size.
    DetectSize = 3,
}

/// Structure-block operating mode.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum StructureBlockMode {
    /// Save mode.
    Save = 0,
    /// Load mode.
    Load = 1,
    /// Corner mode.
    Corner = 2,
    /// Data mode.
    Data = 3,
}

/// Structure mirroring mode.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum StructureMirror {
    /// No mirroring.
    None = 0,
    /// Mirror left to right.
    LeftRight = 1,
    /// Mirror front to back.
    FrontBack = 2,
}

/// Structure rotation mode.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum StructureRotation {
    /// No rotation.
    None = 0,
    /// Clockwise 90 degrees.
    Clockwise90 = 1,
    /// Clockwise 180 degrees.
    Clockwise180 = 2,
    /// Counter-clockwise 90 degrees.
    Counterclockwise90 = 3,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_structure_block",
    id = 0x3C,
    state = Play,
    direction = Serverbound,
)]
/// Programs a structure block at the supplied location.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block)
pub struct ProgramStructureBlock {
    /// Block entity location.
    pub location: Position,
    /// An additional action to perform beyond simply saving the given data.
    pub action: StructureBlockAction,
    /// Structure-block operating mode.
    pub mode: StructureBlockMode,
    /// Structure name.
    pub name: BoundedString<32767>,
    /// X offset, between -48 and 48.
    pub offset_x: Byte,
    /// Y offset, between -48 and 48.
    pub offset_y: Byte,
    /// Z offset, between -48 and 48.
    pub offset_z: Byte,
    /// X size, between 0 and 48.
    pub size_x: Byte,
    /// Y size, between 0 and 48.
    pub size_y: Byte,
    /// Z size, between 0 and 48.
    pub size_z: Byte,
    /// Structure mirroring mode.
    pub mirror: StructureMirror,
    /// Structure rotation mode.
    pub rotation: StructureRotation,
    /// Structure metadata.
    pub metadata: BoundedString<128>,
    /// Integrity, between 0 and 1.
    pub integrity: Float,
    /// Random seed.
    pub seed: VarLong,
    /// 0x01: Ignore entities; 0x02: Show air; 0x04: Show bounding box; 0x08:
    /// Strict placement.
    pub flags: Byte,
}

/// Mode selected by the Set Test Block packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Test_Block)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum TestBlockMode {
    /// Start.
    Start = 0,
    /// Log.
    Log = 1,
    /// Fail.
    Fail = 2,
    /// Accept.
    Accept = 3,
}

#[derive(PacketCodec)]
#[packet(
    name = "set_test_block",
    id = 0x3D,
    state = Play,
    direction = Serverbound,
)]
/// Updates the value of the Test Block at the given position.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Test_Block)
pub struct SetTestBlock {
    /// Test-block position.
    pub position: Position,
    /// Test-block mode.
    pub mode: TestBlockMode,
    /// Message associated with the selected mode.
    pub message: PrefixedString,
}

/// Sign text slot updated by the Update Sign packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Sign)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum SignTextSlot {
    /// Back of the sign.
    Back = 0,
    /// Front of the sign.
    Front = 1,
}

#[derive(PacketCodec)]
#[packet(
    name = "sign_update",
    id = 0x3E,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the Done button is pushed after editing a sign.
///
/// The server only accepts this packet after Open Sign Editor; otherwise it is
/// silently ignored.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Sign)
pub struct UpdateSign {
    /// Block coordinates.
    pub location: Position,
    /// First line of text in the sign.
    pub line_1: BoundedString<384>,
    /// Second line of text in the sign.
    pub line_2: BoundedString<384>,
    /// Third line of text in the sign.
    pub line_3: BoundedString<384>,
    /// Fourth line of text in the sign.
    pub line_4: BoundedString<384>,
    /// Selects which sign text slot is updated.
    pub text_slot: SignTextSlot,
}

#[derive(PacketCodec)]
#[packet(
    name = "spectator_action",
    id = 0x3F,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the player left-clicks in spectator mode.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Spectator_Action)
pub struct SpectatorAction {
    /// If 0, the player was not targeting an entity. Otherwise the ID of the
    /// targeted entity plus 1.
    pub entity_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "teleport_to_entity",
    id = 0x40,
    state = Play,
    direction = Serverbound,
)]
/// Teleports the player to the given entity. The player must be in spectator
/// mode.
///
/// The vanilla client only uses this to teleport to players, but the server
/// appears to accept any type of entity. If the given entity cannot be found
/// or is not loaded, this packet is ignored.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Teleport_To_Entity)
pub struct TeleportToEntity {
    /// UUID of the player to teleport to; can also be an entity UUID.
    pub target_player: Uuid,
}

/// Action performed by the Test Instance Block Action packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum TestInstanceBlockActionType {
    /// Init.
    Init = 0,
    /// Query.
    Query = 1,
    /// Set.
    Set = 2,
    /// Reset.
    Reset = 3,
    /// Save.
    Save = 4,
    /// Export.
    Export = 5,
    /// Run.
    Run = 6,
}

/// Rotation applied to a test instance.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum TestInstanceRotation {
    /// No rotation.
    None = 0,
    /// Clockwise 90 degrees.
    Clockwise90 = 1,
    /// Clockwise 180 degrees.
    Clockwise180 = 2,
    /// Counter-clockwise 90 degrees.
    Counterclockwise90 = 3,
}

/// Status of a test instance.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum TestInstanceStatus {
    /// Cleared.
    Cleared = 0,
    /// Running.
    Running = 1,
    /// Finished.
    Finished = 2,
}

#[derive(PacketCodec)]
#[packet(
    name = "test_instance_block_action",
    id = 0x41,
    state = Play,
    direction = Serverbound,
)]
/// Tries to perform an action on the Test Instance Block at the given
/// position.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action)
pub struct TestInstanceBlockAction {
    /// Test-instance block position.
    pub position: Position,
    /// Action to perform.
    pub action: TestInstanceBlockActionType,
    /// ID in the `minecraft:test_instance` registry.
    pub test: PrefixedOptional<Identifier>,
    /// Test-instance size on the X axis.
    pub size_x: VarInt,
    /// Test-instance size on the Y axis.
    pub size_y: VarInt,
    /// Test-instance size on the Z axis.
    pub size_z: VarInt,
    /// Test-instance rotation.
    pub rotation: TestInstanceRotation,
    /// Whether entities are ignored.
    pub ignore_entities: Boolean,
    /// Current test-instance status.
    pub status: TestInstanceStatus,
    /// Error message shown for the test instance.
    pub error_message: PrefixedOptional<TextComponent>,
}

/// Block face used by the Use Item On packet.
///
/// This enum has the same face ordering as
/// [`BlockFace`](crate::packet::play::BlockFace), but is encoded as a VarInt
/// rather than a Byte.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item_On)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum UseItemFace {
    /// Bottom.
    Bottom = 0,
    /// Top.
    Top = 1,
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
    name = "use_item_on",
    id = 0x42,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the player uses an item on a block.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item_On)
pub struct UseItemOn {
    /// The hand from which the block is placed; 0: main hand, 1: off hand.
    pub hand: InteractionHand,
    /// Block position.
    pub location: Position,
    /// The face on which the block is placed, with the same face ordering as
    /// Player Action.
    pub face: UseItemFace,
    /// The position of the crosshair on the block, from 0 to 1 increasing from
    /// west to east.
    pub cursor_position_x: Float,
    /// The position of the crosshair on the block, from 0 to 1 increasing from
    /// bottom to top.
    pub cursor_position_y: Float,
    /// The position of the crosshair on the block, from 0 to 1 increasing from
    /// north to south.
    pub cursor_position_z: Float,
    /// True when the player's head is inside of a block.
    pub inside_block: Boolean,
    /// Seems to always be false, even when interacting with blocks around or
    /// outside the world border, or while the player is outside the border.
    pub world_border_hit: Boolean,
    /// Block change sequence number.
    pub sequence: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "use_item",
    id = 0x43,
    state = Play,
    direction = Serverbound,
)]
/// Sent when pressing the Use Item key (default: right click) with an item in
/// hand.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item)
pub struct UseItem {
    /// Hand used for the animation. 0: main hand, 1: off hand.
    pub hand: InteractionHand,
    /// Block change sequence number.
    pub sequence: VarInt,
    /// Player head rotation around the Y axis.
    pub yaw: Float,
    /// Player head rotation around the X axis.
    pub pitch: Float,
}

#[derive(PacketCodec)]
#[packet(
    name = "custom_click_action",
    id = 0x44,
    state = Play,
    direction = Serverbound,
)]
/// Sent when the client clicks a text component with the `minecraft:custom`
/// click action. This is meant as an alternative to running a command, but has
/// no effect on vanilla servers.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action)
pub struct CustomClickAction {
    /// The identifier for the click action.
    pub id: Identifier,
    /// Length-prefixed NBT payload, which may be a lone `TAG_End`.
    pub payload: CustomClickPayload,
}
