//! Handshaking state packets.

use mcproto_types::{BoundedString, ProtocolEnum, UnsignedShort, VarInt};

use crate::PacketCodec;

#[derive(ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
/// Selects the protocol state entered after handshaking.
///
/// Intents 2 and 3 both enter Login, but Transfer indicates that another
/// server directed the client to this connection.
pub enum Intent {
    /// [Status](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status)
    Status = 1,
    /// [Login](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login)
    Login = 2,
    /// [Transfer](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer)
    Transfer = 3,
}

/// Server address from the handshake, limited to 255 UTF-16 code units.
pub type ServerAddress = BoundedString<255>;

#[derive(PacketCodec)]
#[packet(
    name = "intention",
    id = 0x00,
    state = Handshaking,
    direction = Serverbound,
)]
/// Causes the server to switch into the target protocol state.
pub struct Handshake {
    /// See [protocol version numbers](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Protocol_version_numbers) (currently 776 in Minecraft 26.2).
    pub protocol_version: VarInt,
    /// Hostname or IP used to connect, limited to 255 UTF-16 code units.
    pub server_address: ServerAddress,
    /// Default is 25565. The vanilla server does not use this information.
    pub server_port: UnsignedShort,
    /// Target state requested by this connection.
    pub intent: Intent,
}
