//! Configuration state packets for protocol 777 (Minecraft Java Edition 26.3).

use std::io::{Read, Write};

use mcproto_codec::{
    error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason},
    io::{read_exact_counted, write_all_counted},
    varint::{VarIntRead, VarIntWrite},
};
use mcproto_types::{
    Boolean, BoundedPrefixedArray, BoundedString, Byte, Either, Identifier, Int, Long, Nbt,
    PrefixedArray, PrefixedOptional, PrefixedString, ProtocolEnum, RemainingBytes, TextComponent,
    TypeCodec, TypeStructCodec, UnsignedByte, Uuid, VarInt,
};

use crate::PacketCodec;

/// Cookie data is limited by vanilla to 5 KiB.
pub type CookiePayload = BoundedPrefixedArray<Byte, 5120>;

/// Locale selected in the client settings.
pub type ClientLocale = BoundedString<16>;

/// SHA-1 resource-pack hash, limited to the protocol field's 40 characters.
pub type ResourcePackHash = BoundedString<40>;

/// Title of a custom crash-report detail.
pub type ReportDetailTitle = BoundedString<128>;

/// Description of a custom crash-report detail.
pub type ReportDetailDescription = BoundedString<4096>;

/// Chat messages shown by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ChatMode {
    /// Display chat messages and commands.
    Enabled = 0,
    /// Display command output only.
    CommandsOnly = 1,
    /// Hide chat messages.
    Hidden = 2,
}

/// Hand selected as the player's main hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum MainHand {
    /// Left hand.
    Left = 0,
    /// Right hand.
    Right = 1,
}

/// Client particle-density preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ParticleStatus {
    /// Display all particles.
    All = 0,
    /// Display fewer particles.
    Decreased = 1,
    /// Display the minimum number of particles.
    Minimal = 2,
}

/// Bit mask of skin layers enabled by the client.
///
/// Unknown bits, including currently unused bit 7, are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DisplayedSkinParts(pub u8);

impl DisplayedSkinParts {
    /// Cape layer.
    pub const CAPE: Self = Self(0x01);
    /// Jacket layer.
    pub const JACKET: Self = Self(0x02);
    /// Left sleeve layer.
    pub const LEFT_SLEEVE: Self = Self(0x04);
    /// Right sleeve layer.
    pub const RIGHT_SLEEVE: Self = Self(0x08);
    /// Left pants-leg layer.
    pub const LEFT_PANTS_LEG: Self = Self(0x10);
    /// Right pants-leg layer.
    pub const RIGHT_PANTS_LEG: Self = Self(0x20);
    /// Hat layer.
    pub const HAT: Self = Self(0x40);

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

impl TypeCodec for DisplayedSkinParts {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        UnsignedByte(self.0).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        UnsignedByte::decode(reader).map(|value| Self(value.0))
    }
}

/// One synchronized registry entry.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct RegistryEntry {
    /// Entry identifier, such as `minecraft:overworld`.
    pub entry_id: Identifier,
    /// Inline entry data, or absent when sourced from a negotiated known pack.
    pub data: PrefixedOptional<Nbt>,
}

/// One tag and the numeric registry entries assigned to it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct RegistryTag {
    /// Tag identifier without the `#` prefix.
    pub name: Identifier,
    /// Numeric IDs replacing the previous contents of this tag.
    pub entries: PrefixedArray<VarInt>,
}

/// Tags belonging to one registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct TaggedRegistry {
    /// Registry identifier, such as `minecraft:block`.
    pub registry: Identifier,
    /// Tags defined for this registry.
    pub tags: PrefixedArray<RegistryTag>,
}

/// A data pack participating in known-pack negotiation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct KnownPack {
    /// Namespace portion of the pack name, such as `minecraft`.
    pub namespace: PrefixedString,
    /// Path portion of the pack name, such as `core`.
    pub id: PrefixedString,
    /// Pack version; for `minecraft:core`, normally the Minecraft version.
    pub version: PrefixedString,
}

/// One key-value entry included in crash and disconnection reports.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct ReportDetail {
    /// Entry title, limited to 128 UTF-16 code units.
    pub title: ReportDetailTitle,
    /// Entry description, limited to 4096 UTF-16 code units.
    pub description: ReportDetailDescription,
}

/// Built-in labels understood by the vanilla server-links screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ServerLinkLabel {
    /// Bug-report link.
    BugReport = 0,
    /// Community-guidelines link.
    CommunityGuidelines = 1,
    /// Support link.
    Support = 2,
    /// Service-status link.
    Status = 3,
    /// Feedback link.
    Feedback = 4,
    /// Community link.
    Community = 5,
    /// Website link.
    Website = 6,
    /// Forums link.
    Forums = 7,
    /// News link.
    News = 8,
    /// Announcements link.
    Announcements = 9,
}

/// A built-in or custom server-link label.
pub type ServerLinkLabelValue = Either<ServerLinkLabel, TextComponent>;

/// One link displayed in the vanilla pause menu.
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct ServerLink {
    /// Built-in label when the Either selector is true, otherwise custom text.
    pub label: ServerLinkLabelValue,
    /// Valid URL opened for this link.
    pub url: PrefixedString,
}

/// Result of handling a server resource pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ResourcePackResult {
    /// The pack was applied successfully.
    SuccessfullyDownloaded = 0,
    /// The player declined the pack.
    Declined = 1,
    /// Downloading the pack failed.
    FailedToDownload = 2,
    /// The player accepted the pack request.
    Accepted = 3,
    /// The pack finished downloading.
    Downloaded = 4,
    /// The supplied URL was invalid.
    InvalidUrl = 5,
    /// Reloading resources failed.
    FailedToReload = 6,
    /// The request was discarded.
    Discarded = 7,
}

/// Length-prefixed NBT payload carried by a custom click action.
///
/// The protocol also permits a lone `TAG_End` byte to represent no NBT value.
#[derive(Debug, Clone, PartialEq)]
pub enum CustomClickPayload {
    /// A lone `TAG_End` (`0x00`).
    End,
    /// A complete network-NBT value.
    Nbt(Nbt),
}

impl TypeCodec for CustomClickPayload {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        let encoded = match self {
            Self::End => vec![0],
            Self::Nbt(value) => {
                let mut bytes = Vec::new();
                value.encode(&mut bytes)?;
                bytes
            }
        };
        let length = i32::try_from(encoded.len()).map_err(|_| {
            CodecError::invalid_encoding_for_operation(
                CodecKind::Nbt,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::LengthOutOfRange {
                    max: i32::MAX as usize,
                    actual: encoded.len(),
                },
            )
        })?;
        let prefix_size = writer
            .write_varint_with_size(length)
            .map_err(|error| error.with_context(CodecKind::Nbt))?;
        write_all_counted(writer, &encoded, CodecKind::Nbt, prefix_size)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let (length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::Nbt))?;
        if length < 0 {
            return Err(CodecError::invalid_encoding(
                CodecKind::Nbt,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: length },
            ));
        }

        let mut encoded = vec![0; length as usize];
        read_exact_counted(reader, &mut encoded, CodecKind::Nbt, prefix_size)?;
        if encoded == [0] {
            return Ok(Self::End);
        }

        let mut input = encoded.as_slice();
        let value = Nbt::decode(&mut input)?;
        if !input.is_empty() {
            return Err(CodecError::invalid_encoding(
                CodecKind::Nbt,
                prefix_size + encoded.len(),
                InvalidEncodingReason::InvalidNbt,
            ));
        }
        Ok(Self::Nbt(value))
    }
}

#[derive(PacketCodec)]
#[packet(name = "cookie_request", id = 0x00, state = Configuration, direction = Clientbound)]
/// Requests a cookie previously stored by the client.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request)
pub struct CookieRequest {
    /// Identifier of the requested cookie.
    pub key: Identifier,
}

#[derive(PacketCodec)]
#[packet(name = "custom_payload", id = 0x01, state = Configuration, direction = Clientbound)]
/// Carries mod or plugin data selected by a plugin-channel identifier.
///
/// The payload has no universal length prefix and occupies the rest of the
/// packet body. Its internal format is defined by `channel`.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound))
pub struct PluginMessageClientbound<T: TypeCodec = RemainingBytes> {
    /// Plugin channel naming the payload format.
    pub channel: Identifier,
    /// Channel-specific payload.
    pub data: T,
}

#[derive(PacketCodec)]
#[packet(name = "disconnect", id = 0x02, state = Configuration, direction = Clientbound)]
/// Disconnects the client during configuration.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect)
pub struct Disconnect {
    /// Reason shown to the player.
    pub reason: TextComponent,
}

#[derive(PacketCodec)]
#[packet(name = "finish_configuration", id = 0x03, state = Configuration, direction = Clientbound)]
/// Notifies the client that configuration data is complete.
///
/// After validating registries and tags, the client responds with
/// [`AcknowledgeFinishConfiguration`] and both sides enter the Play state.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Finish_Configuration)
pub struct FinishConfiguration;

#[derive(PacketCodec)]
#[packet(name = "keep_alive", id = 0x04, state = Configuration, direction = Clientbound)]
/// Requests a keep-alive response carrying the same ID.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound))
pub struct KeepAliveClientbound {
    /// Random keep-alive identifier generated by the server.
    pub keep_alive_id: Long,
}

#[derive(PacketCodec)]
#[packet(name = "ping", id = 0x05, state = Configuration, direction = Clientbound)]
/// Requests a [`Pong`] carrying the same ID.
///
/// The vanilla server does not normally send this packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping)
pub struct Ping {
    /// Ping identifier.
    pub id: Int,
}

#[derive(PacketCodec)]
#[packet(name = "reset_chat", id = 0x06, state = Configuration, direction = Clientbound)]
/// Resets the client's chat state.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Reset_Chat)
pub struct ResetChat;

#[derive(PacketCodec)]
#[packet(name = "registry_data", id = 0x07, state = Configuration, direction = Clientbound)]
/// Supplies one synchronized registry sourced from server data packs.
///
/// Entry order defines numeric IDs beginning at zero. The client accumulates
/// these packets and validates them when configuration finishes.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Registry_Data)
pub struct RegistryData {
    /// Registry identifier, such as `minecraft:dimension_type`.
    pub registry_id: Identifier,
    /// Registry entries in numeric-ID order.
    pub entries: PrefixedArray<RegistryEntry>,
}

#[derive(PacketCodec)]
#[packet(name = "resource_pack_pop", id = 0x08, state = Configuration, direction = Clientbound)]
/// Removes one resource pack, or all packs when no UUID is present.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Resource_Pack)
pub struct RemoveResourcePack {
    /// Pack UUID; absent removes every resource pack.
    pub uuid: PrefixedOptional<Uuid>,
}

#[derive(PacketCodec)]
#[packet(name = "resource_pack_push", id = 0x09, state = Configuration, direction = Clientbound)]
/// Offers a resource pack to the client.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Add_Resource_Pack)
pub struct AddResourcePack {
    /// Unique identifier of the resource pack.
    pub uuid: Uuid,
    /// URL from which the pack is downloaded.
    pub url: PrefixedString,
    /// Case-insensitive SHA-1 hash; malformed values disable hash verification.
    pub hash: ResourcePackHash,
    /// Whether declining the pack causes disconnection.
    pub forced: Boolean,
    /// Optional text shown in the accept-or-decline prompt.
    pub prompt_message: PrefixedOptional<TextComponent>,
}

#[derive(PacketCodec)]
#[packet(name = "store_cookie", id = 0x0a, state = Configuration, direction = Clientbound)]
/// Stores data on the client for later connections and server transfers.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Store_Cookie)
pub struct StoreCookie {
    /// Cookie identifier.
    pub key: Identifier,
    /// Cookie data, limited by vanilla to 5 KiB.
    pub payload: CookiePayload,
}

#[derive(PacketCodec)]
#[packet(name = "transfer", id = 0x0b, state = Configuration, direction = Clientbound)]
/// Directs the client to reconnect to another server with Transfer intent.
///
/// Cookies remain available across the transfer.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer)
pub struct Transfer {
    /// Destination hostname or IP address.
    pub host: PrefixedString,
    /// Destination port encoded as a VarInt.
    pub port: VarInt,
}

#[derive(PacketCodec)]
#[packet(name = "update_enabled_features", id = 0x0c, state = Configuration, direction = Clientbound)]
/// Enables the listed vanilla or experimental feature flags.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Feature_Flags)
pub struct FeatureFlags {
    /// Enabled feature identifiers; `minecraft:vanilla` enables vanilla features.
    pub feature_flags: PrefixedArray<Identifier>,
}

#[derive(PacketCodec)]
#[packet(name = "update_tags", id = 0x0d, state = Configuration, direction = Clientbound)]
/// Replaces tag contents for the supplied registries.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Tags)
pub struct UpdateTags {
    /// Registries and their tag definitions.
    pub tagged_registries: PrefixedArray<TaggedRegistry>,
}

#[derive(PacketCodec)]
#[packet(name = "select_known_packs", id = 0x0e, state = Configuration, direction = Clientbound)]
/// Lists data packs present on the server for known-pack negotiation.
///
/// The client responds with the known subset in the same order.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(clientbound))
pub struct KnownPacksClientbound {
    /// Packs available on the server.
    pub known_packs: PrefixedArray<KnownPack>,
}

#[derive(PacketCodec)]
#[packet(name = "custom_report_details", id = 0x0f, state = Configuration, direction = Clientbound)]
/// Adds key-value text entries to crash and disconnection reports.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Report_Details)
pub struct CustomReportDetails {
    /// At most 32 custom report entries.
    pub details: BoundedPrefixedArray<ReportDetail, 32>,
}

#[derive(PacketCodec)]
#[packet(name = "server_links", id = 0x10, state = Configuration, direction = Clientbound)]
/// Supplies links displayed in the vanilla pause menu.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Links)
pub struct ServerLinks {
    /// Built-in or custom-labeled server links.
    pub links: PrefixedArray<ServerLink>,
}

#[derive(PacketCodec)]
#[packet(name = "clear_dialog", id = 0x11, state = Configuration, direction = Clientbound)]
/// Closes the current dialog and returns to the previous screen.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Dialog)
pub struct ClearDialog;

#[derive(PacketCodec)]
#[packet(name = "show_dialog", id = 0x12, state = Configuration, direction = Clientbound)]
/// Displays a custom dialog defined inline as NBT.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Show_Dialog_(configuration))
pub struct ShowDialog {
    /// Inline dialog definition.
    pub dialog: Nbt,
}

#[derive(PacketCodec)]
#[packet(name = "code_of_conduct", id = 0x13, state = Configuration, direction = Clientbound)]
/// Shows the server's code of conduct and waits for acceptance.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Code_of_Conduct)
pub struct CodeOfConduct {
    /// Server code-of-conduct text.
    pub code_of_conduct: PrefixedString,
}

#[derive(PacketCodec)]
#[packet(name = "client_information", id = 0x00, state = Configuration, direction = Serverbound)]
/// Reports client settings when connecting or whenever they change.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information)
pub struct ClientInformation {
    /// Client locale, such as `en_GB`.
    pub locale: ClientLocale,
    /// Client render distance in chunks.
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

#[derive(PacketCodec)]
#[packet(name = "cookie_response", id = 0x01, state = Configuration, direction = Serverbound)]
/// Responds to a [`CookieRequest`].
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response)
pub struct CookieResponse {
    /// Identifier of the requested cookie.
    pub key: Identifier,
    /// Stored cookie data, or absent when no matching cookie exists.
    pub payload: PrefixedOptional<CookiePayload>,
}

#[derive(PacketCodec)]
#[packet(name = "custom_payload", id = 0x02, state = Configuration, direction = Serverbound)]
/// Carries mod or plugin data selected by a plugin-channel identifier.
///
/// The payload has no universal length prefix and occupies the rest of the
/// packet body. Its internal format is defined by `channel`.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound))
pub struct PluginMessageServerbound<T: TypeCodec = RemainingBytes> {
    /// Plugin channel naming the payload format.
    pub channel: Identifier,
    /// Channel-specific payload.
    pub data: T,
}

#[derive(PacketCodec)]
#[packet(name = "finish_configuration", id = 0x03, state = Configuration, direction = Serverbound)]
/// Confirms that the client validated configuration and is ready for Play.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Finish_Configuration)
pub struct AcknowledgeFinishConfiguration;

#[derive(PacketCodec)]
#[packet(name = "keep_alive", id = 0x04, state = Configuration, direction = Serverbound)]
/// Responds to [`KeepAliveClientbound`] with the same ID.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound))
pub struct KeepAliveServerbound {
    /// Keep-alive ID copied from the clientbound request.
    pub keep_alive_id: Long,
}

#[derive(PacketCodec)]
#[packet(name = "pong", id = 0x05, state = Configuration, direction = Serverbound)]
/// Responds to [`Ping`] with the same ID.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong)
pub struct Pong {
    /// Ping ID copied from the request.
    pub id: Int,
}

#[derive(PacketCodec)]
#[packet(name = "resource_pack", id = 0x06, state = Configuration, direction = Serverbound)]
/// Reports the client's handling of a resource-pack request.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response)
pub struct ResourcePackResponse {
    /// UUID from the corresponding [`AddResourcePack`] request.
    pub uuid: Uuid,
    /// Current result of processing the resource pack.
    pub result: ResourcePackResult,
}

#[derive(PacketCodec)]
#[packet(name = "select_known_packs", id = 0x07, state = Configuration, direction = Serverbound)]
/// Reports the ordered subset of server packs also known to the client.
///
/// The server may omit inline registry NBT sourced from these packs.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(serverbound))
pub struct KnownPacksServerbound {
    /// Packs shared by the server and client.
    pub known_packs: PrefixedArray<KnownPack>,
}

#[derive(PacketCodec)]
#[packet(name = "custom_click_action", id = 0x08, state = Configuration, direction = Serverbound)]
/// Reports activation of a text component's `minecraft:custom` click action.
///
/// Vanilla servers ignore this packet.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action)
pub struct CustomClickAction {
    /// Identifier assigned to the custom click action.
    pub id: Identifier,
    /// Length-prefixed NBT payload, which may be a lone `TAG_End`.
    pub payload: CustomClickPayload,
}

#[derive(PacketCodec)]
#[packet(name = "accept_code_of_conduct", id = 0x09, state = Configuration, direction = Serverbound)]
/// Acknowledges the server's [`CodeOfConduct`].
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Accept_Code_of_Conduct)
pub struct AcceptCodeOfConduct;
