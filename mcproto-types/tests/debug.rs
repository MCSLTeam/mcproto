//! Protocol tests for debug subscription events and updates.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    BeeDebugData, BeeHiveDebugData, Boolean, BreezeDebugData, DebugGoalName, DebugPathNode,
    DebugPathNodeType, DebugStructureInfo, DebugStructurePiece, DebugSubscriptionData,
    DebugSubscriptionEvent, DebugSubscriptionType, DebugSubscriptionUpdate, Double,
    EntityBlockIntersectionDebugData, EntityBlockIntersectionState, EntityPathDebugData, Float,
    GameEventDebugData, GameEventListenerDebugData, GoalSelectorDebugData, Int,
    NeighborUpdateDebugData, PointOfInterestDebugData, Position, PrefixedArray, PrefixedOptional,
    PrefixedString, RaidDebugData, RedstoneWireOrientationDebugData, StructureDebugData, TypeCodec,
    VarInt, VillagerBrainDebugData,
};

fn position(x: i32, y: i16, z: i32) -> Position {
    Position { x, y, z }
}

fn path_node() -> DebugPathNode {
    DebugPathNode {
        x: Int(1),
        y: Int(2),
        z: Int(3),
        walked_distance: Float(4.5),
        cost_malus: Float(0.25),
        closed: Boolean(true),
        node_type: DebugPathNodeType::WalkableDoor,
        f: Float(9.0),
    }
}

fn every_payload() -> Vec<DebugSubscriptionData> {
    let node = path_node();
    let piece = DebugStructurePiece {
        bounding_box_min: position(-5, 0, -5),
        bounding_box_max: position(5, 10, 5),
        is_start: Boolean(true),
    };

    vec![
        DebugSubscriptionData::DedicatedServerTickTime,
        DebugSubscriptionData::Bee(BeeDebugData {
            hive_position: PrefixedOptional::some(position(1, 2, 3)),
            flower_position: PrefixedOptional::none(),
            travel_ticks: VarInt(12),
            blacklisted_hives: PrefixedArray(vec![position(-1, 4, 8)]),
        }),
        DebugSubscriptionData::VillagerBrain(VillagerBrainDebugData {
            name: PrefixedString("Nitwit".into()),
            profession: PrefixedString("none".into()),
            xp: Int(7),
            health: Float(18.0),
            max_health: Float(20.0),
            inventory: PrefixedString("[]".into()),
            wants_golem: Boolean(false),
            anger_level: Int(0),
            activities: PrefixedArray(vec![PrefixedString("idle".into())]),
            behaviors: PrefixedArray(vec![PrefixedString("look".into())]),
            memories: PrefixedArray(vec![PrefixedString("home".into())]),
            gossips: PrefixedArray(vec![PrefixedString("minor_positive".into())]),
            pois: PrefixedArray(vec![position(2, 64, 2)]),
            potential_pois: PrefixedArray(vec![position(3, 64, 3)]),
        }),
        DebugSubscriptionData::Breeze(BreezeDebugData {
            attack_target: PrefixedOptional::some(VarInt(42)),
            jump_target: PrefixedOptional::some(position(8, 70, 8)),
        }),
        DebugSubscriptionData::GoalSelector(GoalSelectorDebugData {
            priority: VarInt(1),
            is_running: Boolean(true),
            name: DebugGoalName::new("minecraft:random_stroll").unwrap(),
        }),
        DebugSubscriptionData::EntityPath(EntityPathDebugData {
            reached: Boolean(false),
            next_block_index: Int(2),
            block_position: position(4, 65, 4),
            nodes: PrefixedArray(vec![node.clone()]),
            target_nodes: PrefixedArray(vec![node.clone()]),
            open_set: PrefixedArray(vec![node.clone()]),
            closed_set: PrefixedArray(vec![node]),
            max_node_distance: Float(32.0),
        }),
        DebugSubscriptionData::EntityBlockIntersection(EntityBlockIntersectionDebugData {
            state: EntityBlockIntersectionState::InFluid,
        }),
        DebugSubscriptionData::BeeHive(BeeHiveDebugData {
            block_type: VarInt(25),
            occupant_count: VarInt(2),
            honey_level: VarInt(5),
            sedated: Boolean(true),
        }),
        DebugSubscriptionData::PointOfInterest(PointOfInterestDebugData {
            position: position(10, 64, -10),
            poi_type: VarInt(3),
            free_ticket_count: VarInt(1),
        }),
        DebugSubscriptionData::RedstoneWireOrientation(RedstoneWireOrientationDebugData {
            id: VarInt(17),
        }),
        DebugSubscriptionData::VillageSection,
        DebugSubscriptionData::Raid(RaidDebugData {
            positions: PrefixedArray(vec![position(100, 70, 100)]),
        }),
        DebugSubscriptionData::Structure(StructureDebugData {
            structures: PrefixedArray(vec![DebugStructureInfo {
                bounding_box_min: position(-16, 0, -16),
                bounding_box_max: position(16, 100, 16),
                pieces: PrefixedArray(vec![piece]),
            }]),
        }),
        DebugSubscriptionData::GameEventListener(GameEventListenerDebugData {
            listener_radius: VarInt(16),
        }),
        DebugSubscriptionData::NeighborUpdate(NeighborUpdateDebugData {
            position: position(0, 64, 0),
        }),
        DebugSubscriptionData::GameEvent(GameEventDebugData {
            event: VarInt(6),
            x: Double(1.25),
            y: Double(64.0),
            z: Double(-3.5),
        }),
    ]
}

#[test]
fn all_sixteen_event_payloads_roundtrip() {
    for (expected_type, data) in every_payload().into_iter().enumerate() {
        let event = DebugSubscriptionEvent::new(data);
        let mut encoded = Vec::new();
        event.encode(&mut encoded).unwrap();

        assert_eq!(encoded[0], expected_type as u8);
        let mut input = encoded.as_slice();
        assert_eq!(DebugSubscriptionEvent::decode(&mut input).unwrap(), event);
        assert!(input.is_empty());
    }
}

#[test]
fn event_and_update_have_the_documented_wire_layout() {
    let event = DebugSubscriptionEvent::new(DebugSubscriptionData::EntityBlockIntersection(
        EntityBlockIntersectionDebugData {
            state: EntityBlockIntersectionState::InAir,
        },
    ));
    let mut encoded = Vec::new();
    event.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [6, 2]);

    let absent = DebugSubscriptionUpdate::absent(DebugSubscriptionType::Bee);
    let mut encoded = Vec::new();
    absent.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [1, 0]);
    assert!(!absent.is_present());

    let present = DebugSubscriptionUpdate::present(event.data);
    let mut encoded = Vec::new();
    present.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [6, 1, 2]);
    assert!(present.is_present());

    let mut input = encoded.as_slice();
    assert_eq!(
        DebugSubscriptionUpdate::decode(&mut input).unwrap(),
        present
    );
    assert!(input.is_empty());
}

#[test]
fn present_fieldless_update_still_writes_its_boolean() {
    let value = DebugSubscriptionUpdate::present(DebugSubscriptionData::DedicatedServerTickTime);
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0, 1]);
    assert_eq!(
        DebugSubscriptionUpdate::decode(&mut encoded.as_slice()).unwrap(),
        value
    );
}

#[test]
fn invalid_subscription_and_payload_enums_are_rejected() {
    let error = DebugSubscriptionEvent::decode(&mut [16].as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 16 })
    );
    assert_eq!(error.contexts(), &[CodecKind::DebugSubscriptionEvent]);

    let error = DebugSubscriptionEvent::decode(&mut [6, 3].as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::Enum);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 3 })
    );
    assert_eq!(
        error.contexts(),
        &[
            CodecKind::DebugSubscriptionData,
            CodecKind::DebugSubscriptionEvent,
        ]
    );
}

#[test]
fn goal_name_limit_counts_utf16_code_units() {
    let accepted = DebugGoalName::new(format!("{}a", "😀".repeat(127))).unwrap();
    assert_eq!(accepted.as_str().encode_utf16().count(), 255);

    let error = DebugGoalName::new("😀".repeat(128)).unwrap_err();
    assert_eq!(error.actual_code_units, 256);
}
