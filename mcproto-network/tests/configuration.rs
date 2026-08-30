//! Configuration packet protocol tests for Minecraft Java Edition 26.2.

use mcproto_network::{
    DecodePacket, Direction, EncodePacket, Packet, ProtocolState,
    packet::configuration::{
        AcceptCodeOfConduct, AcknowledgeFinishConfiguration, ClientInformation, CodeOfConduct,
        CookieRequest, CookieResponse, CustomClickAction, CustomClickPayload, CustomReportDetails,
        Disconnect, FeatureFlags, FinishConfiguration, KeepAliveClientbound, KeepAliveServerbound,
        KnownPacksClientbound, KnownPacksServerbound, Ping, PluginMessageClientbound,
        PluginMessageServerbound, Pong, RegistryData, RemoveResourcePack, ResourcePackResponse,
        ServerLinks, ShowDialog, StoreCookie, Transfer, UpdateTags,
    },
};
use mcproto_types::{Identifier, RemainingBytes, TypeCodec};

fn assert_packet<P: Packet>(id: i32, direction: Direction) {
    assert_eq!(P::ID.get(), id);
    assert_eq!(P::STATE, ProtocolState::Configuration);
    assert_eq!(P::DIRECTION, direction);
}

#[test]
fn configuration_packet_ids_match_protocol_776() {
    assert_packet::<CookieRequest>(0x00, Direction::Clientbound);
    assert_packet::<PluginMessageClientbound>(0x01, Direction::Clientbound);
    assert_packet::<Disconnect>(0x02, Direction::Clientbound);
    assert_packet::<FinishConfiguration>(0x03, Direction::Clientbound);
    assert_packet::<KeepAliveClientbound>(0x04, Direction::Clientbound);
    assert_packet::<Ping>(0x05, Direction::Clientbound);
    assert_packet::<mcproto_network::packet::configuration::ResetChat>(
        0x06,
        Direction::Clientbound,
    );
    assert_packet::<RegistryData>(0x07, Direction::Clientbound);
    assert_packet::<RemoveResourcePack>(0x08, Direction::Clientbound);
    assert_packet::<mcproto_network::packet::configuration::AddResourcePack>(
        0x09,
        Direction::Clientbound,
    );
    assert_packet::<StoreCookie>(0x0a, Direction::Clientbound);
    assert_packet::<Transfer>(0x0b, Direction::Clientbound);
    assert_packet::<FeatureFlags>(0x0c, Direction::Clientbound);
    assert_packet::<UpdateTags>(0x0d, Direction::Clientbound);
    assert_packet::<KnownPacksClientbound>(0x0e, Direction::Clientbound);
    assert_packet::<CustomReportDetails>(0x0f, Direction::Clientbound);
    assert_packet::<ServerLinks>(0x10, Direction::Clientbound);
    assert_packet::<mcproto_network::packet::configuration::ClearDialog>(
        0x11,
        Direction::Clientbound,
    );
    assert_packet::<ShowDialog>(0x12, Direction::Clientbound);
    assert_packet::<CodeOfConduct>(0x13, Direction::Clientbound);

    assert_packet::<ClientInformation>(0x00, Direction::Serverbound);
    assert_packet::<CookieResponse>(0x01, Direction::Serverbound);
    assert_packet::<PluginMessageServerbound>(0x02, Direction::Serverbound);
    assert_packet::<AcknowledgeFinishConfiguration>(0x03, Direction::Serverbound);
    assert_packet::<KeepAliveServerbound>(0x04, Direction::Serverbound);
    assert_packet::<Pong>(0x05, Direction::Serverbound);
    assert_packet::<ResourcePackResponse>(0x06, Direction::Serverbound);
    assert_packet::<KnownPacksServerbound>(0x07, Direction::Serverbound);
    assert_packet::<CustomClickAction>(0x08, Direction::Serverbound);
    assert_packet::<AcceptCodeOfConduct>(0x09, Direction::Serverbound);
}

#[test]
fn plugin_payload_consumes_the_packet_body_without_a_prefix() {
    let channel = Identifier::new("example:test").unwrap();
    let mut body = Vec::new();
    channel.encode(&mut body).unwrap();
    body.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

    let mut input = body.as_slice();
    let packet = PluginMessageClientbound::<RemainingBytes>::decode_body(&mut input).unwrap();

    assert_eq!(packet.channel, channel);
    assert_eq!(packet.data, RemainingBytes(vec![0xde, 0xad, 0xbe, 0xef]));
    assert!(input.is_empty());
}

#[test]
fn custom_click_end_payload_includes_its_size_prefix() {
    let packet = CustomClickAction {
        id: Identifier::new("example:click").unwrap(),
        payload: CustomClickPayload::End,
    };
    let mut body = Vec::new();

    packet.encode_body(&mut body).unwrap();

    assert_eq!(&body[body.len() - 2..], [0x01, 0x00]);
}

#[test]
fn custom_click_payload_respects_its_declared_size() {
    let mut input = [0x01, 0x00, 0xaa].as_slice();

    assert_eq!(
        CustomClickPayload::decode(&mut input).unwrap(),
        CustomClickPayload::End
    );
    assert_eq!(input, [0xaa]);
}
