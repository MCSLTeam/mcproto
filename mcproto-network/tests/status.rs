//! Status packet protocol tests.

use mcproto_network::{
    DecodePacket, Direction, Packet, PacketName, ProtocolState, packet::status::StatusResponse,
};
use mcproto_types::{Content, JsonComponent, PrefixedString, TypeCodec};

const EXAMPLE: &str = r#"{
    "version": {
        "name": "1.21.8",
        "protocol": 772
    },
    "players": {
        "max": 20,
        "online": 1,
        "sample": [
            {
                "name": "thinkofdeath",
                "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
            }
        ]
    },
    "description": {
        "text": "Hello, world!"
    },
    "favicon": "data:image/png;base64,<data>",
    "enforcesSecureChat": false
}"#;

#[test]
fn status_response_decodes_official_example() {
    let mut encoded = Vec::new();
    PrefixedString(EXAMPLE.into()).encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();

    let packet = StatusResponse::decode_body(&mut input).unwrap();

    assert!(input.is_empty());
    assert_eq!(StatusResponse::NAME, PacketName::new("status_response"));
    assert_eq!(StatusResponse::ID.get(), 0x00);
    assert_eq!(StatusResponse::STATE, ProtocolState::Status);
    assert_eq!(StatusResponse::DIRECTION, Direction::Clientbound);

    let response = packet.json_response;
    assert_eq!(response.version.name.as_deref(), Some("1.21.8"));
    assert_eq!(response.version.protocol, 772);
    let players = response.players.unwrap();
    assert_eq!((players.online, players.max), (1, 20));
    let sample = players.sample.unwrap();
    assert_eq!(sample[0].name, "thinkofdeath");
    assert_eq!(
        sample[0].id.unwrap().to_string(),
        "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
    );
    assert_eq!(response.enforces_secure_chat, Some(false));
    assert_eq!(
        response.favicon.as_deref(),
        Some("data:image/png;base64,<data>")
    );
    let description = response.description.unwrap();
    assert!(matches!(
        description,
        JsonComponent::Object(object)
            if matches!(object.content, Content::Text { ref text } if text == "Hello, world!")
    ));
}

#[test]
fn status_response_accepts_optional_fields_being_absent() {
    let mut encoded = Vec::new();
    PrefixedString(r#"{"version":{"protocol":772}}"#.into())
        .encode(&mut encoded)
        .unwrap();

    let packet = StatusResponse::decode_body(&mut encoded.as_slice()).unwrap();

    assert_eq!(packet.json_response.version.name, None);
    assert_eq!(packet.json_response.players, None);
    assert_eq!(packet.json_response.description, None);
    assert_eq!(packet.json_response.favicon, None);
    assert_eq!(packet.json_response.enforces_secure_chat, None);
}
