//! Serverbound Play packet tests for Minecraft Java Edition 26.3.

use mcproto_network::{
    Direction, EncodePacket, Packet, ProtocolState,
    packet::configuration::CustomClickPayload,
    packet::play::{
        ChangeRecipeBookSettings, CustomClickAction, GameRule, PlayerInput, PlayerLoaded, Pong,
        ProgramCommandBlock, ProgramCommandBlockMinecart, ProgramJigsawBlock,
        ProgramStructureBlock, Punch, RenameItem, ResourcePackResponse, SeenAdvancements,
        SeenAdvancementsAction, SelectTrade, SetBeaconEffect, SetCreativeModeSlot, SetGameRules,
        SetHeldItemServerbound, SetSeenRecipe, SetTestBlock, SpectatorAction, TeleportToEntity,
        TestInstanceBlockAction, UpdateSign, UseItem, UseItemOn,
    },
};
use mcproto_types::{BoundedString, Identifier, Optional, PrefixedArray, TypeCodec};

fn assert_packet<P: Packet>(id: i32, name: &str) {
    assert_eq!(P::ID.get(), id);
    assert_eq!(P::NAME.as_str(), name);
    assert_eq!(P::STATE, ProtocolState::Play);
    assert_eq!(P::DIRECTION, Direction::Serverbound);
}

#[test]
fn serverbound_play_packets_match_protocol_777() {
    assert_packet::<PlayerInput>(0x2b, "player_input");
    assert_packet::<PlayerLoaded>(0x2c, "player_loaded");
    assert_packet::<Pong>(0x2d, "pong");
    assert_packet::<Punch>(0x2e, "punch");
    assert_packet::<ChangeRecipeBookSettings>(0x2f, "recipe_book_change_settings");
    assert_packet::<SetSeenRecipe>(0x30, "recipe_book_seen_recipe");
    assert_packet::<RenameItem>(0x31, "rename_item");
    assert_packet::<ResourcePackResponse>(0x32, "resource_pack");
    assert_packet::<SeenAdvancements>(0x33, "seen_advancements");
    assert_packet::<SelectTrade>(0x34, "select_trade");
    assert_packet::<SetBeaconEffect>(0x35, "set_beacon");
    assert_packet::<SetHeldItemServerbound>(0x36, "set_carried_item");
    assert_packet::<ProgramCommandBlock>(0x37, "set_command_block");
    assert_packet::<ProgramCommandBlockMinecart>(0x38, "set_command_minecart");
    assert_packet::<SetCreativeModeSlot>(0x39, "set_creative_mode_slot");
    assert_packet::<SetGameRules>(0x3a, "set_game_rule");
    assert_packet::<ProgramJigsawBlock>(0x3b, "set_jigsaw_block");
    assert_packet::<ProgramStructureBlock>(0x3c, "set_structure_block");
    assert_packet::<SetTestBlock>(0x3d, "set_test_block");
    assert_packet::<UpdateSign>(0x3e, "sign_update");
    assert_packet::<SpectatorAction>(0x3f, "spectator_action");
    assert_packet::<TeleportToEntity>(0x40, "teleport_to_entity");
    assert_packet::<TestInstanceBlockAction>(0x41, "test_instance_block_action");
    assert_packet::<UseItemOn>(0x42, "use_item_on");
    assert_packet::<UseItem>(0x43, "use_item");
    assert_packet::<CustomClickAction>(0x44, "custom_click_action");
}

#[test]
fn seen_advancements_encodes_tab_only_when_opened() {
    let tab = Identifier::new("minecraft:story").unwrap();
    let opened = SeenAdvancements {
        action: SeenAdvancementsAction::OpenedTab,
        tab_id: Optional::some(tab.clone()),
    };
    let mut body = Vec::new();
    opened.encode_body(&mut body).unwrap();

    let mut expected = vec![0];
    tab.encode(&mut expected).unwrap();
    assert_eq!(body, expected);

    let closed = SeenAdvancements {
        action: SeenAdvancementsAction::ClosedScreen,
        tab_id: Optional::none(),
    };
    let mut body = Vec::new();
    closed.encode_body(&mut body).unwrap();
    assert_eq!(body, [1]);
}

#[test]
fn game_rules_use_the_documented_entry_order() {
    let name = Identifier::new("minecraft:do_daylight_cycle").unwrap();
    let value = BoundedString::<32767>::new("false").unwrap();
    let packet = SetGameRules {
        rules: PrefixedArray(vec![GameRule {
            name: name.clone(),
            value: value.clone(),
        }]),
    };
    let mut body = Vec::new();
    packet.encode_body(&mut body).unwrap();

    let mut expected = vec![1];
    name.encode(&mut expected).unwrap();
    value.encode(&mut expected).unwrap();
    assert_eq!(body, expected);
}

#[test]
fn custom_click_action_reuses_the_length_prefixed_nbt_payload() {
    let id = Identifier::new("example:callback").unwrap();
    let packet = CustomClickAction {
        id: id.clone(),
        payload: CustomClickPayload::End,
    };
    let mut body = Vec::new();
    packet.encode_body(&mut body).unwrap();

    let mut expected = Vec::new();
    id.encode(&mut expected).unwrap();
    expected.extend_from_slice(&[1, 0]);
    assert_eq!(body, expected);
}
