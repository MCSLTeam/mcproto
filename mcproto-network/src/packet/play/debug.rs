//! Debug packet definitions for protocol 777 (Minecraft Java Edition 26.3).

use mcproto_types::{
    DebugSubscriptionEvent, DebugSubscriptionUpdate, Int, Long, Position, PrefixedArray,
    ProtocolEnum, VarInt,
};

use crate::PacketCodec;

/// Sample type used by the Debug Sample packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Sample)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum DebugSampleType {
    /// Four different tick-related metrics, each one represented by one long
    /// on the array. They are measured in nano-seconds, and are as follows:
    ///
    /// - 0: Full tick time: Aggregate of the three times below;
    /// - 1: Server tick time: Main server tick logic;
    /// - 2: Tasks time: Tasks scheduled to execute after the main logic;
    /// - 3: Idle time: Time idling to complete the full 50ms tick cycle.
    ///
    /// Note that the vanilla client calculates the timings used for min/max/
    /// average display by subtracting the idle time from the full tick time.
    /// This can cause the displayed values to go negative if the idle time is
    /// (nonsensically) greater than the full tick time.
    TickTime = 0,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_block_value",
    id = 0x1A,
    state = Play,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Block_Value)
pub struct DebugBlockValue {
    /// Location.
    pub location: Position,
    /// Update.
    pub update: DebugSubscriptionUpdate,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_chunk_value",
    id = 0x1B,
    state = Play,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Chunk_Value)
///
/// Note: The order of X and Z is inverted, because the client reads them as
/// one big-endian Long, with Z being the upper 32 bits.
pub struct DebugChunkValue {
    /// Chunk Z.
    pub chunk_z: Int,
    /// Chunk X.
    pub chunk_x: Int,
    /// Update.
    pub update: DebugSubscriptionUpdate,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_entity_value",
    id = 0x1C,
    state = Play,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Entity_Value)
pub struct DebugEntityValue {
    /// Entity ID.
    pub entity_id: VarInt,
    /// Update.
    pub update: DebugSubscriptionUpdate,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_event",
    id = 0x1D,
    state = Play,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Event)
pub struct DebugEvent {
    /// Event.
    pub event: DebugSubscriptionEvent,
}

#[derive(PacketCodec)]
#[packet(
    name = "debug_sample",
    id = 0x1E,
    state = Play,
    direction = Clientbound,
)]
/// Sample data that is sent periodically after the client has subscribed with
/// the [Debug Sample Subscription](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Sample_Subscription).
///
/// The vanilla server only sends debug samples to players who are server
/// operators.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Sample)
pub struct DebugSample {
    /// Array of type-dependent samples.
    pub sample: PrefixedArray<Long>,
    /// See below.
    pub sample_type: DebugSampleType,
}
