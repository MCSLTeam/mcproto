//! Login state packets.

use mcproto_types::{
    Boolean, BoundedPrefixedArray, Byte, GameProfile, Identifier, JsonTextComponent, PrefixedArray,
    PrefixedOptional, PrefixedString, TypeCodec, Uuid, VarInt,
};

use crate::PacketCodec;

#[derive(PacketCodec)]
#[packet(
    name = "hello",
    id = 0x00,
    state = Login,
    direction = Serverbound,
)]

/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Start)
pub struct LoginStart {
    /// Player's Username. Some plugins allow player to use longer name.
    pub username: PrefixedString,
    /// The [UUID](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Type:UUID) of the player logging in. Unused by the vanilla server.
    pub player_uuid: Uuid,
}

#[derive(PacketCodec)]
#[packet(
    name = "login_disconnect",
    id = 0x00,
    state = Login,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect_(login))
pub struct Disconnect {
    /// The reason why the player was disconnected.
    pub reason: JsonTextComponent,
}

#[derive(PacketCodec)]
#[packet(
    name = "hello",
    id = 0x01,
    state = Login,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Encryption_Request)
///
/// Details: [protocol encryption](https://minecraft.wiki/w/Protocol_encryption)
pub struct EncryptionRequest {
    /// Always empty when sent by the vanilla server.
    pub server_id: PrefixedString,
    /// The server's public key, in bytes.
    pub public_key: PrefixedArray<Byte>,
    /// The nonce used to verify the encryption response.
    pub verify_token: PrefixedArray<Byte>,
    /// Whether the client should attempt to [authenticate through mojang servers](https://minecraft.wiki/w/Java_Edition_protocol/Encryption#Authentication).
    pub authenticate: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "key",
    id = 0x01,
    state = Login,
    direction = Serverbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Encryption_Response)
///
/// Details: [protocol encryption](https://minecraft.wiki/w/Protocol_encryption)
pub struct EncryptionResponse {
    ///  Shared Secret value, encrypted with the server's public key.
    pub shared_secret: PrefixedArray<Byte>,
    ///  Verify Token value, encrypted with the same public key as the shared secret.
    pub verify_token: PrefixedArray<Byte>,
}

#[derive(PacketCodec)]
#[packet(
    name = "custom_query_answer",
    id = 0x02,
    state = Login,
    direction = Serverbound,
)]
/// [wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Plugin_Response)
pub struct LoginPluginResponse<T: TypeCodec> {
    /// Should match ID from server.
    pub message_id: VarInt,
    /// Any data, depending on the channel. Only present if the client understood the request. Typically this would be a sequence of fields using standard data types, but some unofficial channels have unusual formats. There is no length prefix that applies to all channel types,
    ///
    /// but the format specific to the channel may or may not include one or more length prefixes (e.g. for strings).
    pub data: PrefixedOptional<T>,
}

#[derive(PacketCodec)]
#[packet(
    name = "login_finished",
    id = 0x02,
    state = Login,
    direction = Clientbound,
)]
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Success)
pub struct LoginSuccess {
    /// No notes in wiki.
    pub profile: GameProfile,
    /// No notes in wiki.
    pub session_id: Uuid,
}

#[derive(PacketCodec)]
#[packet(
    name = "login_compression",
    id = 0x03,
    state = Login,
    direction = Serverbound,
)]
/// Enables compression.
///
/// If compression is enabled, all following packets are encoded in the [compressed packet format](https://minecraft.wiki/w/Java_Edition_protocol/Packets#With_compression).
///
/// Negative values will disable compression, meaning the packet format should remain in the uncompressed packet format. However, this packet is entirely optional, and if not sent, compression will also not be enabled (the vanilla server does not send the packet when compression is disabled).
pub struct SetCompression {
    /// Maximum size of a packet before it is compressed.
    pub threshold: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "login_acknowledged",
    id = 0x03,
    state = Login,
    direction = Serverbound,
)]
/// Acknowledgement to the [Login Success](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Success) packet sent by the server.
pub struct LoginAcknowledged;

#[derive(PacketCodec)]
#[packet(
    name = "custom_query",
    id = 0x04,
    state = Login,
    direction = Clientbound,
)]
/// Used to implement a custom handshaking flow together with Login Plugin Response.
///
/// Unlike plugin messages in "play" mode, these messages follow a lock-step request/response scheme, where the client is expected to respond to a request indicating whether it understood.
///
/// The vanilla client always responds that it hasn't understood and sends an empty payload.

pub struct LoginPluginRequest<T: TypeCodec> {
    /// Generated by the server - should be unique to the connection.
    pub message_id: VarInt,
    /// Name of the [plugin channel](https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels) used to send the data.
    pub channel: Identifier,
    /// Any data, depending on the channel. Only present if the client understood the request. Typically this would be a sequence of fields using standard data types, but some unofficial channels have unusual formats. There is no length prefix that applies to all channel types,
    ///
    /// but the format specific to the channel may or may not include one or more length prefixes (e.g. for strings).
    pub data: T,
}

#[derive(PacketCodec)]
#[packet(
    name = "cookie_response",
    id = 0x04,
    state = Login,
    direction = Serverbound,
)]
/// Response to a [Cookie Request](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request) from the server.
///
/// The vanilla server only accepts responses of up to 5 kiB in size.

pub struct CookieResponse {
    /// The identifier of the cookie.
    pub key: Identifier,
    /// The data of the cookie.
    pub data: PrefixedOptional<BoundedPrefixedArray<Byte, 5120>>,
}

#[derive(PacketCodec)]
#[packet(
    name = "cookie_request",
    id = 0x05,
    state = Login,
    direction = Clientbound,
)]
/// Requests a cookie that was previously stored.
pub struct CookieRequest {
    /// The identifier of the cookie.
    pub key: Identifier,
}
