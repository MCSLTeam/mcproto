use std::collections::{BTreeMap, HashMap};

use mcproto_codec::error::{CodecErrorKind, CodecOperation, InvalidEncodingReason};
use mcproto_codec::varint::VarIntWrite;
use mcproto_types::{
    TypeCodec,
    component::{
        ClickEvent, CommandString, Component, ComponentObject, Content, DialogReference,
        HoverEvent, HttpUrl, JsonComponent, NbtComponent, NbtDisplay, NbtTarget, ObjectContent,
        PlayerModel, PlayerName, PlayerProfile, PositiveI32, Profile, ProfileProperty,
        ProfilePropertyName, ResourceLocation, RgbColor, ShadowColor,
        TEXT_COMPONENT_FORMAT_VERSION, TextColor, Uuid,
    },
    json_text_component::JsonTextComponent,
    text_component::{NbtValue, TextComponent},
};
use serde_json::{Value as JsonValue, json};

fn resource(value: &str) -> ResourceLocation {
    ResourceLocation::new(value).unwrap()
}

fn json_roundtrip(component: JsonComponent) -> JsonValue {
    let encoded = serde_json::to_value(&component).unwrap();
    let decoded: JsonComponent = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, component);
    encoded
}

fn component_with_click(event: ClickEvent<JsonValue>) -> JsonComponent {
    let mut object = ComponentObject::text("clickable");
    object.style.click_event = Some(event);
    Component::Object(Box::new(object))
}

fn component_with_hover(event: HoverEvent<JsonValue>) -> JsonComponent {
    let mut object = ComponentObject::text("hoverable");
    object.style.hover_event = Some(event);
    Component::Object(Box::new(object))
}

#[test]
fn all_click_event_variants_roundtrip() {
    let inline_dialog = BTreeMap::from([
        ("type".to_owned(), json!("notice")),
        ("title".to_owned(), json!({ "text": "Hello" })),
    ]);
    let cases = [
        (
            "open_url",
            ClickEvent::OpenUrl {
                url: HttpUrl::new("https://example.com/path").unwrap(),
            },
        ),
        (
            "open_file",
            ClickEvent::OpenFile {
                path: "screenshots/example.png".to_owned(),
            },
        ),
        (
            "run_command",
            ClickEvent::RunCommand {
                command: CommandString::new("say hello").unwrap(),
            },
        ),
        (
            "suggest_command",
            ClickEvent::SuggestCommand {
                command: CommandString::new("msg Steve hello").unwrap(),
            },
        ),
        (
            "change_page",
            ClickEvent::ChangePage {
                page: PositiveI32::new(2).unwrap(),
            },
        ),
        (
            "copy_to_clipboard",
            ClickEvent::CopyToClipboard {
                value: "copied text".to_owned(),
            },
        ),
        (
            "show_dialog",
            ClickEvent::ShowDialog {
                dialog: DialogReference::Id(resource("minecraft:welcome")),
            },
        ),
        (
            "show_dialog",
            ClickEvent::ShowDialog {
                dialog: DialogReference::Inline(inline_dialog),
            },
        ),
        (
            "custom",
            ClickEvent::Custom {
                id: resource("example:callback"),
                payload: Some(json!({ "nested": [1, true, { "key": "value" }] })),
            },
        ),
    ];

    for (expected_action, event) in cases {
        let encoded = json_roundtrip(component_with_click(event));
        assert_eq!(
            encoded.pointer("/click_event/action"),
            Some(&json!(expected_action))
        );
    }
}

#[test]
fn all_hover_event_variants_roundtrip() {
    let cases = [
        (
            "show_text",
            HoverEvent::ShowText {
                value: Box::new(Component::text("tooltip")),
            },
        ),
        (
            "show_item",
            HoverEvent::ShowItem {
                id: resource("minecraft:diamond_sword"),
                count: Some(2),
                components: BTreeMap::from([(
                    resource("minecraft:custom_name"),
                    json!({ "text": "Blade" }),
                )]),
            },
        ),
        (
            "show_entity",
            HoverEvent::ShowEntity {
                name: Some(Box::new(Component::text("Steve"))),
                id: resource("minecraft:player"),
                uuid: Uuid::parse("00112233-4455-6677-8899-aabbccddeeff").unwrap(),
            },
        ),
    ];

    for (expected_action, event) in cases {
        let encoded = json_roundtrip(component_with_hover(event));
        assert_eq!(
            encoded.pointer("/hover_event/action"),
            Some(&json!(expected_action))
        );
    }
}

#[test]
fn content_and_object_variants_roundtrip() {
    assert_eq!(TEXT_COMPONENT_FORMAT_VERSION, "26.1");

    let profile = Profile {
        name: Some(PlayerName::new("Test_Player").unwrap()),
        id: Some(Uuid::parse("00112233-4455-6677-8899-aabbccddeeff").unwrap()),
        properties: vec![ProfileProperty {
            name: ProfilePropertyName::Textures,
            value: "base64-texture".to_owned(),
            signature: Some("signature".to_owned()),
        }],
        texture: Some(resource("example:skin")),
        cape: Some(resource("example:cape")),
        elytra: Some(resource("example:elytra")),
        model: Some(PlayerModel::Slim),
    };
    let components: Vec<JsonComponent> = vec![
        Component::object(Content::Text {
            text: "plain".to_owned(),
        }),
        Component::object(Content::Translatable {
            translate: "chat.type.text".to_owned(),
            fallback: Some("<%s> %s".to_owned()),
            with: vec![Component::text("Steve"), Component::text("Hello")],
        }),
        Component::object(Content::Score {
            score: mcproto_types::component::Score {
                name: "Steve".to_owned(),
                objective: "points".to_owned(),
            },
        }),
        Component::object(Content::Selector {
            selector: "@a".to_owned(),
            separator: Some(Box::new(Component::text(" | "))),
        }),
        Component::object(Content::Keybind {
            keybind: "key.inventory".to_owned(),
        }),
        Component::object(Content::Nbt {
            nbt: "Pos[0]".to_owned(),
            target: NbtTarget::Entity("@s".to_owned()),
            display: NbtDisplay::Styled,
            separator: None,
        }),
        Component::object(Content::Nbt {
            nbt: "Items".to_owned(),
            target: NbtTarget::Block("~ ~ ~".to_owned()),
            display: NbtDisplay::Plain,
            separator: Some(Box::new(Component::text(", "))),
        }),
        Component::object(Content::Nbt {
            nbt: "value".to_owned(),
            target: NbtTarget::Storage(resource("example:data")),
            display: NbtDisplay::Interpret,
            separator: None,
        }),
        Component::object(Content::Object {
            object: ObjectContent::Atlas {
                atlas: Some(resource("minecraft:blocks")),
                sprite: resource("minecraft:block/stone"),
            },
            fallback: Some("[stone]".to_owned()),
        }),
        Component::object(Content::Object {
            object: ObjectContent::Player {
                player: PlayerProfile::Name(PlayerName::new("Steve").unwrap()),
                hat: Some(true),
            },
            fallback: None,
        }),
        Component::object(Content::Object {
            object: ObjectContent::Player {
                player: PlayerProfile::Profile(profile),
                hat: Some(false),
            },
            fallback: Some("[player]".to_owned()),
        }),
    ];

    for component in components {
        json_roundtrip(component);
    }
}

#[test]
fn uuid_and_shadow_color_representations_are_accepted() {
    let expected = Uuid::parse("00112233-4455-6677-8899-aabbccddeeff").unwrap();
    let compact: Uuid = serde_json::from_value(json!("00112233445566778899aabbccddeeff")).unwrap();
    let list: Uuid = serde_json::from_value(json!([
        0x0011_2233,
        0x4455_6677,
        0x8899_aabbu32 as i32,
        0xccdd_eeffu32 as i32
    ]))
    .unwrap();
    let int_array: Uuid = fastnbt::from_value(&NbtValue::IntArray(fastnbt::IntArray::new(vec![
        0x0011_2233,
        0x4455_6677,
        0x8899_aabbu32 as i32,
        0xccdd_eeffu32 as i32,
    ])))
    .unwrap();
    assert_eq!(compact, expected);
    assert_eq!(list, expected);
    assert_eq!(int_array, expected);
    assert_eq!(
        serde_json::to_value(expected).unwrap(),
        json!(expected.to_string())
    );
    assert!(Uuid::parse("not-a-uuid").is_err());
    assert!(
        fastnbt::from_value::<Uuid>(&NbtValue::IntArray(fastnbt::IntArray::new(vec![1, 2, 3])))
            .is_err()
    );

    let rgba: ShadowColor = serde_json::from_value(json!([1.0, 0.5, 0.0, 0.25])).unwrap();
    assert_eq!(rgba.argb(), 0x40ff_8000);
    let argb: ShadowColor = serde_json::from_value(json!(-1)).unwrap();
    assert_eq!(argb.argb(), -1);
    assert!(serde_json::from_value::<ShadowColor>(json!([1.1, 0.0, 0.0, 1.0])).is_err());
}

#[test]
fn rgb_colors_are_bounded_and_roundtrip_without_truncation() {
    let rgb = RgbColor::new(0x12_ab_ef).unwrap();
    assert_eq!(rgb.value(), 0x12_ab_ef);
    assert_eq!(rgb.channels(), [0x12, 0xab, 0xef]);
    assert_eq!(RgbColor::from_channels(0x12, 0xab, 0xef), rgb);
    assert!(RgbColor::new(RgbColor::MAX + 1).is_err());
    assert!(TextColor::rgb(RgbColor::MAX + 1).is_err());

    let color = TextColor::Rgb(rgb);
    let encoded = serde_json::to_value(color).unwrap();
    assert_eq!(encoded, json!("#12abef"));
    assert_eq!(serde_json::from_value::<TextColor>(encoded).unwrap(), color);
    assert!(serde_json::from_value::<TextColor>(json!("#1234567")).is_err());
}

#[test]
fn malformed_schema_objects_are_rejected() {
    let invalid = [
        json!([]),
        json!({ "bold": true }),
        json!({ "type": "object", "object": "atlas" }),
        json!({ "text": "x", "click_event": { "action": "open_url" } }),
        json!({ "text": "x", "hover_event": { "action": "show_text" } }),
        json!({
            "nbt": "Pos",
            "entity": "@s",
            "plain": true,
            "interpret": true
        }),
    ];

    for value in invalid {
        assert!(
            serde_json::from_value::<JsonComponent>(value.clone()).is_err(),
            "accepted {value}"
        );
    }
}

#[test]
fn nbt_dynamic_payload_and_item_components_roundtrip() {
    let mut object = ComponentObject::text("dynamic");
    object.style.click_event = Some(ClickEvent::Custom {
        id: resource("example:callback"),
        payload: Some(NbtValue::Compound(HashMap::from([
            ("enabled".to_owned(), NbtValue::Byte(1)),
            (
                "values".to_owned(),
                NbtValue::List(vec![NbtValue::Int(1), NbtValue::Int(2)]),
            ),
        ]))),
    });
    object.style.hover_event = Some(HoverEvent::ShowItem {
        id: resource("minecraft:stone"),
        count: Some(3),
        components: BTreeMap::from([(
            resource("minecraft:custom_data"),
            NbtValue::Compound(HashMap::from([(
                "source".to_owned(),
                NbtValue::String("test".to_owned()),
            )])),
        )]),
    });
    let component = TextComponent(NbtComponent::Object(Box::new(object)));

    let mut encoded = Vec::new();
    component.encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();
    assert_eq!(TextComponent::decode(&mut input).unwrap(), component);
    assert!(input.is_empty());
}

fn nested_component<V>(depth: usize) -> Component<V> {
    assert!(depth > 0);
    let mut component = Component::text("leaf");
    for _ in 1..depth {
        let mut parent = ComponentObject::text("parent");
        parent.extra.push(component);
        component = Component::Object(Box::new(parent));
    }
    component
}

#[test]
fn component_depth_limit_is_checked_before_writing() {
    let boundary = JsonTextComponent(nested_component(512));
    let mut encoded = Vec::new();
    boundary.encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();
    assert_eq!(JsonTextComponent::decode(&mut input).unwrap(), boundary);
    assert!(input.is_empty());

    let mut json_output = Vec::new();
    let error = JsonTextComponent(nested_component(513))
        .encode(&mut json_output)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidJson)
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert!(json_output.is_empty());

    let mut nbt_output = Vec::new();
    let error = TextComponent(nested_component(513))
        .encode(&mut nbt_output)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert!(nbt_output.is_empty());
}

fn nested_json_value(depth: usize) -> JsonValue {
    let mut value = JsonValue::Null;
    for _ in 0..depth {
        value = JsonValue::Array(vec![value]);
    }
    value
}

fn component_with_custom_payload<V>(payload: V) -> Component<V> {
    let mut object = ComponentObject::text("dynamic");
    object.style.click_event = Some(ClickEvent::Custom {
        id: resource("example:callback"),
        payload: Some(payload),
    });
    Component::Object(Box::new(object))
}

#[test]
fn deeply_nested_dynamic_payloads_are_rejected_before_serialization() {
    let mut json_output = Vec::new();
    let error = JsonTextComponent(component_with_custom_payload(nested_json_value(513)))
        .encode(&mut json_output)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidJson)
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert!(json_output.is_empty());

    let mut nbt_value = NbtValue::Byte(0);
    for _ in 0..513 {
        nbt_value = NbtValue::List(vec![nbt_value]);
    }
    let mut nbt_output = Vec::new();
    let error = TextComponent(component_with_custom_payload(nbt_value))
        .encode(&mut nbt_output)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidNbt)
    );
    assert_eq!(error.operation(), CodecOperation::Write);
    assert!(nbt_output.is_empty());
}

#[test]
fn deeply_nested_json_payload_is_rejected_during_decode_without_stack_overflow() {
    let arrays = 513;
    let json = format!(
        "{{\"text\":\"dynamic\",\"click_event\":{{\"action\":\"custom\",\"id\":\"example:callback\",\"payload\":{}null{}}}}}",
        "[".repeat(arrays),
        "]".repeat(arrays)
    );
    let mut encoded = Vec::new();
    encoded.write_varint(json.len() as i32).unwrap();
    encoded.extend_from_slice(json.as_bytes());

    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidJson)
    );
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}

#[test]
fn extreme_json_depth_is_rejected_before_deserialization() {
    let arrays = 2_048;
    let json = format!("{}null{}", "[".repeat(arrays), "]".repeat(arrays));
    let mut encoded = Vec::new();
    encoded.write_varint(json.len() as i32).unwrap();
    encoded.extend_from_slice(json.as_bytes());

    let error = JsonTextComponent::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidJson)
    );
    assert_eq!(error.operation(), CodecOperation::Read);
    assert_eq!(error.bytes_processed(), encoded.len());
}
