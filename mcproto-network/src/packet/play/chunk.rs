//! Chunk packet definitions.

use mcproto_types::{Byte, Float, Int, PrefixedArray, TypeStructCodec, VarInt};

use crate::PacketCodec;

#[derive(PacketCodec)]
#[packet(
    name = "chunk_batch_finished",
    id = 0x0B,
    state = Play,
    direction = Clientbound,
)]
/// Marks the end of a chunk batch. The vanilla client marks the time it
/// receives this packet and calculates the elapsed duration since the
/// [beginning of the chunk batch](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Start).
/// The client uses this duration and the batch size received in this packet to
/// estimate the number of milliseconds elapsed per chunk received. This value
/// is then used to calculate the desired number of chunks per tick through the
/// formula `25 / millisPerChunk`, which is reported to the server through
/// [Chunk Batch Received](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Received).
/// This likely uses `25` instead of the normal tick duration of `50` so chunk
/// processing will only use half of the client's and network's bandwidth.
///
/// The vanilla client uses the samples from the latest 15 batches to estimate
/// the milliseconds per chunk number.
pub struct ChunkBatchFinished {
    /// Number of chunks.
    pub batch_size: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "chunk_batch_received",
    id = 0x0B,
    state = Play,
    direction = Serverbound,
)]
/// Notifies the server that the chunk batch has been received by the client.
/// The server uses the value sent in this packet to adjust the number of
/// chunks to be sent in a batch.
///
/// The vanilla server will stop sending further chunk data until the client
/// acknowledges the sent chunk batch. After the first acknowledgement, the
/// server adjusts this number to allow up to 10 unacknowledged batches.
pub struct ChunkBatchReceived {
    /// Desired chunks per tick.
    pub chunks_per_tick: Float,
}
#[derive(PacketCodec)]
#[packet(
    name = "chunk_batch_start",
    id = 0x0C,
    state = Play,
    direction = Clientbound,
)]
/// Marks the start of a chunk batch. The vanilla client marks and stores the
/// time it receives this packet.
pub struct ChunkBatchStart;

/// One chunk's biome data inside the Chunk Biomes packet.
#[derive(Debug, Clone, PartialEq, Eq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct ChunkBiomeData {
    /// Chunk coordinate (block coordinate divided by 16, rounded down)
    pub chunk_z: Int,
    /// Chunk coordinate (block coordinate divided by 16, rounded down)
    pub chunk_x: Int,
    /// Chunk data structure, with sections containing only the `Biomes` field
    pub data: PrefixedArray<Byte>,
}

#[derive(PacketCodec)]
#[packet(
    name = "chunks_biomes",
    id = 0x0D,
    state = Play,
    direction = Clientbound,
)]
/// Note: The order of X and Z is inverted, because the client reads them as
/// one big-endian `Long`, with Z being the upper 32 bits.
pub struct ChunkBiomes {
    /// Chunk biome data.
    pub chunk_biome_data: PrefixedArray<ChunkBiomeData>,
}
