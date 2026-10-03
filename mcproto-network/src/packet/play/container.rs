//! Container packet definitions for protocol 777 (Minecraft Java Edition 26.3).

use mcproto_types::{
    Boolean, BoundedPrefixedArray, Byte, HashedSlot, PrefixedArray, ProtocolEnum, Short, Slot,
    TypeStructCodec, VarInt,
};

use crate::PacketCodec;

#[derive(PacketCodec)]
#[packet(
    name = "container_close",
    id = 0x11,
    state = Play,
    direction = Clientbound,
)]
/// This packet is sent from the server to the client when a window is
/// forcibly closed, such as when a chest is destroyed while it's open. The
/// vanilla client disregards the provided window ID and closes any active
/// window.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Close_Container)
pub struct CloseContainerClientbound {
    /// This is the ID of the window that was closed. 0 for inventory.
    pub window_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_close",
    id = 0x13,
    state = Play,
    direction = Serverbound,
)]
/// This packet is sent by the client when closing a window.
///
/// Vanilla clients send a Close Window packet with Window ID 0 to close their
/// inventory, even though there is never an Open Screen packet for the
/// inventory.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Close_Container)
pub struct CloseContainerServerbound {
    /// This is the ID of the window that was closed. 0 for player inventory.
    pub window_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_button_click",
    id = 0x11,
    state = Play,
    direction = Serverbound,
)]
/// Used when clicking on window buttons. Until 1.14, this was only used by
/// enchantment tables.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container_Button)
pub struct ClickContainerButton {
    /// The ID of the window sent by Open Screen.
    pub window_id: VarInt,
    /// Meaning depends on window type.
    pub button_id: VarInt,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_set_content",
    id = 0x12,
    state = Play,
    direction = Clientbound,
)]
/// Replaces the contents of a container window. Sent by the server upon
/// initialization of a container window or the player's inventory, and in
/// response to state ID mismatches (see Click Container).
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Content)
pub struct SetContainerContent {
    /// The ID of window which items are being sent for. 0 for player
    /// inventory. The client ignores any packets targeting a Window ID other
    /// than the current one. However, an exception is made for the player
    /// inventory, which may be targeted at any time. (The vanilla server does
    /// not appear to utilize this special case.)
    pub window_id: VarInt,
    /// A server-managed sequence number used to avoid desynchronization; see
    /// Click Container.
    pub state_id: VarInt,
    /// The contents of every slot in the window.
    pub slot_data: PrefixedArray<Slot>,
    /// Item being dragged with the mouse.
    pub carried_item: Slot,
}

/// Inventory operation mode of a Click Container packet.
///
/// The distinct type of click performed by the client is determined by the
/// combination of the Mode and Button fields.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ContainerClickMode {
    /// Left or right mouse click.
    Normal = 0,
    /// Shift + left or right mouse click.
    ShiftClick = 1,
    /// Number key 1 through 9, or the offhand swap key.
    NumberKey = 2,
    /// Middle click, only defined for creative players in non-player
    /// inventories.
    MiddleClick = 3,
    /// Drop key (Q) or Control + Drop key.
    Drop = 4,
    /// "Painting mode" drag operation.
    Drag = 5,
    /// Double click.
    DoubleClick = 6,
}

/// One changed slot inside a Click Container packet.
#[derive(Debug, Clone, PartialEq, Eq, TypeStructCodec)]
#[type_struct_codec(kind = TypeStruct)]
pub struct ContainerChangedSlot {
    /// The slot that was changed.
    pub slot_number: Short,
    /// New data for this slot, in the client's opinion.
    pub slot_data: HashedSlot,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_click",
    id = 0x12,
    state = Play,
    direction = Serverbound,
)]
/// This packet is sent by the client when the player clicks on a slot in a
/// window.
///
/// After performing the action, the server compares the results to the slot
/// change information included in the packet, as applied on top of the
/// server's view of the container's state prior to the action. For any slots
/// that do not match, it sends Set Container Slot packets containing the
/// correct results. If State ID does not match the last ID sent by the
/// server, it will instead send a full Set Container Content to resynchronize
/// the client.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container)
pub struct ClickContainer {
    /// The ID of the window that was clicked. 0 for player inventory. The
    /// server ignores any packets targeting a Window ID other than the
    /// current one, including ignoring 0 when any other window is open.
    pub window_id: VarInt,
    /// The last received State ID from either a Set Container Slot or a Set
    /// Container Content packet.
    pub state_id: VarInt,
    /// The clicked slot number.
    pub slot: Short,
    /// The button used in the click.
    pub button: Byte,
    /// Inventory operation mode.
    pub mode: ContainerClickMode,
    /// New data for each slot changed by this click.
    pub changed_slots: BoundedPrefixedArray<ContainerChangedSlot, 128>,
    /// Item carried by the cursor.
    pub carried_item: HashedSlot,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_slot_state_changed",
    id = 0x14,
    state = Play,
    direction = Serverbound,
)]
/// This packet is sent by the client when toggling the state of a Crafter.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Container_Slot_State)
pub struct ChangeContainerSlotState {
    /// This is the ID of the slot that was changed.
    pub slot_id: VarInt,
    /// This is the ID of the window that was changed.
    pub window_id: VarInt,
    /// The new state of the slot. True for enabled, false for disabled.
    pub state: Boolean,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_set_slot",
    id = 0x14,
    state = Play,
    direction = Clientbound,
)]
/// Sent by the server when an item in a slot (in a window) is
/// added/removed.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Slot)
pub struct SetContainerSlot {
    /// The window that is being updated. 0 for player inventory. The client
    /// ignores any packets targeting a Window ID other than the current one.
    pub window_id: VarInt,
    /// A server-managed sequence number used to avoid desynchronization; see
    /// Click Container.
    pub state_id: VarInt,
    /// The slot that should be updated.
    pub slot: Short,
    /// The new contents of the slot.
    pub slot_data: Slot,
}

#[derive(PacketCodec)]
#[packet(
    name = "container_set_data",
    id = 0x13,
    state = Play,
    direction = Clientbound,
)]
/// This packet is used to inform the client that part of a GUI window should
/// be updated.
///
/// The meaning of the Property field depends on the type of the window.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Property)
pub struct SetContainerProperty {
    /// The window being updated.
    pub window_id: VarInt,
    /// The property to be updated.
    pub property: Short,
    /// The new value for the property.
    pub value: Short,
}
