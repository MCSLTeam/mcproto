//! Protocol tests for the Entity Metadata Format.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::*;

fn assert_roundtrip<T>(value: T, expected_prefix: &[u8])
where
    T: TypeCodec + std::fmt::Debug + PartialEq,
{
    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert!(
        encoded.starts_with(expected_prefix),
        "encoded={encoded:02x?}"
    );

    let mut input = encoded.as_slice();
    assert_eq!(T::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn metadata_sequence_uses_indices_and_ff_terminator() {
    let metadata = EntityMetadata::new(vec![
        EntityMetadataEntry::try_new(0, EntityMetadataValue::Byte(Byte(0x20))).unwrap(),
        EntityMetadataEntry::try_new(7, EntityMetadataValue::Boolean(Boolean(true))).unwrap(),
        EntityMetadataEntry::try_new(
            EntityMetadataIndex::MAX,
            EntityMetadataValue::Direction(Direction::East),
        )
        .unwrap(),
    ])
    .unwrap();

    let mut encoded = Vec::new();
    metadata.encode(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        [
            0x00, 0x00, 0x20, // Index 0, Byte, value
            0x07, 0x08, 0x01, // Index 7, Boolean, true
            0xfe, 0x0c, 0x05, // Index 254, Direction, East
            0xff, // Terminator
        ]
    );

    let mut input = encoded.as_slice();
    assert_eq!(EntityMetadata::decode(&mut input).unwrap(), metadata);
    assert!(input.is_empty());
}

#[test]
fn empty_metadata_consumes_only_its_terminator() {
    let mut encoded = Vec::new();
    EntityMetadata::default().encode(&mut encoded).unwrap();
    assert_eq!(encoded, [0xff]);

    let mut input = [0xff, 0xaa].as_slice();
    assert_eq!(
        EntityMetadata::decode(&mut input).unwrap(),
        EntityMetadata::default()
    );
    assert_eq!(input, [0xaa]);
}

#[test]
fn reserved_and_duplicate_indices_are_rejected() {
    let error = EntityMetadataEntry::try_new(
        EntityMetadataIndex::TERMINATOR,
        EntityMetadataValue::Byte(Byte(0)),
    )
    .unwrap_err();
    assert_eq!(error.index(), 0xff);

    let first = EntityMetadataEntry::try_new(3, EntityMetadataValue::Byte(Byte(0))).unwrap();
    let second = EntityMetadataEntry::try_new(3, EntityMetadataValue::VarInt(VarInt(1))).unwrap();
    let error = EntityMetadata::new(vec![first, second]).unwrap_err();
    assert_eq!(error.index().get(), 3);

    let encoded = [
        0x03, 0x00, 0x00, // First index 3
        0x03, 0x01, 0x01, // Duplicate index 3
        0xff,
    ];
    let error = EntityMetadata::decode(&mut encoded.as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::EntityMetadata);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::DuplicateEntityMetadataIndex {
            index: 3
        })
    );
}

#[test]
fn all_43_metadata_value_types_roundtrip_with_their_documented_ids() {
    let values = vec![
        (0, EntityMetadataValue::Byte(Byte(-7))),
        (1, EntityMetadataValue::VarInt(VarInt(300))),
        (2, EntityMetadataValue::VarLong(VarLong(40_000))),
        (3, EntityMetadataValue::Float(Float(1.5))),
        (
            4,
            EntityMetadataValue::String(PrefixedString("metadata".into())),
        ),
        (
            5,
            EntityMetadataValue::TextComponent(TextComponent::text("name")),
        ),
        (
            6,
            EntityMetadataValue::OptionalTextComponent(OptionalTextComponent::none()),
        ),
        (7, EntityMetadataValue::Slot(Slot::Empty)),
        (8, EntityMetadataValue::Boolean(Boolean(true))),
        (
            9,
            EntityMetadataValue::Rotations(Rotations {
                x: Float(1.0),
                y: Float(2.0),
                z: Float(3.0),
            }),
        ),
        (
            10,
            EntityMetadataValue::Position(Position { x: 1, y: 2, z: 3 }),
        ),
        (
            11,
            EntityMetadataValue::OptionalPosition(OptionalPosition::none()),
        ),
        (12, EntityMetadataValue::Direction(Direction::North)),
        (
            13,
            EntityMetadataValue::OptionalLivingEntityReference(
                OptionalLivingEntityReference::none(),
            ),
        ),
        (
            14,
            EntityMetadataValue::BlockState(BlockStateId::new(42).unwrap()),
        ),
        (
            15,
            EntityMetadataValue::OptionalBlockState(OptionalBlockState::Absent),
        ),
        (16, EntityMetadataValue::Particle(Particle::AngryVillager)),
        (17, EntityMetadataValue::Particles(Particles::default())),
        (
            18,
            EntityMetadataValue::VillagerData(VillagerData {
                villager_type: VillagerType::Plains,
                profession: VillagerProfession::Librarian,
                level: VarInt(4),
            }),
        ),
        (
            19,
            EntityMetadataValue::OptionalVarInt(OptionalVarInt::some(41).unwrap()),
        ),
        (20, EntityMetadataValue::Pose(Pose::Swimming)),
        (
            21,
            EntityMetadataValue::CatVariant(CatVariantId::new(1).unwrap()),
        ),
        (
            22,
            EntityMetadataValue::CatSoundVariant(CatSoundVariantId::new(2).unwrap()),
        ),
        (
            23,
            EntityMetadataValue::CowVariant(CowVariantId::new(3).unwrap()),
        ),
        (
            24,
            EntityMetadataValue::CowSoundVariant(CowSoundVariantId::new(4).unwrap()),
        ),
        (
            25,
            EntityMetadataValue::WolfVariant(WolfVariantId::new(5).unwrap()),
        ),
        (
            26,
            EntityMetadataValue::WolfSoundVariant(WolfSoundVariantId::new(6).unwrap()),
        ),
        (
            27,
            EntityMetadataValue::FrogVariant(FrogVariantId::new(7).unwrap()),
        ),
        (
            28,
            EntityMetadataValue::PigVariant(PigVariantId::new(8).unwrap()),
        ),
        (
            29,
            EntityMetadataValue::PigSoundVariant(PigSoundVariantId::new(9).unwrap()),
        ),
        (
            30,
            EntityMetadataValue::ChickenVariant(ChickenVariantId::new(10).unwrap()),
        ),
        (
            31,
            EntityMetadataValue::ChickenSoundVariant(ChickenSoundVariantId::new(11).unwrap()),
        ),
        (
            32,
            EntityMetadataValue::ZombieNautilusVariant(ZombieNautilusVariantId::new(12).unwrap()),
        ),
        (
            33,
            EntityMetadataValue::OptionalGlobalPosition(OptionalGlobalPosition::none()),
        ),
        (34, EntityMetadataValue::PaintingVariant(IdOr::id(0))),
        (
            35,
            EntityMetadataValue::SnifferState(SnifferState::Searching),
        ),
        (
            36,
            EntityMetadataValue::ArmadilloState(ArmadilloState::Scared),
        ),
        (
            37,
            EntityMetadataValue::CopperGolemState(CopperGolemState::DroppingItem),
        ),
        (
            38,
            EntityMetadataValue::WeatheringCopperState(WeatheringCopperState::Weathered),
        ),
        (
            39,
            EntityMetadataValue::Vector3(Vector3 {
                x: Float(1.0),
                y: Float(2.0),
                z: Float(3.0),
            }),
        ),
        (
            40,
            EntityMetadataValue::Quaternion(Quaternion {
                x: Float(0.0),
                y: Float(0.0),
                z: Float(0.0),
                w: Float(1.0),
            }),
        ),
        (
            41,
            EntityMetadataValue::ResolvableProfile(ResolvableProfile::partial(
                PartialProfile::default(),
            )),
        ),
        (42, EntityMetadataValue::HumanoidArm(HumanoidArm::Right)),
    ];

    assert_eq!(values.len(), 43);
    for (expected_id, value) in values {
        assert_eq!(value.type_id(), expected_id);
        assert_roundtrip(value, &[expected_id as u8]);
    }
}

#[test]
fn every_data_bearing_particle_layout_roundtrips() {
    let state = || BlockStateId::new(5).unwrap();
    let particles = vec![
        (
            1,
            Particle::Block(BlockParticleData {
                block_state: state(),
            }),
        ),
        (
            2,
            Particle::BlockMarker(BlockMarkerParticleData {
                block_state: state(),
            }),
        ),
        (
            7,
            Particle::Geyser(GeyserParticleData {
                water_blocks: Int(1),
            }),
        ),
        (
            8,
            Particle::GeyserBase(GeyserBaseParticleData {
                water_blocks: Int(2),
                burst_impulse_base: Float(0.25),
            }),
        ),
        (
            9,
            Particle::GeyserPoof(GeyserPoofParticleData {
                water_blocks: Int(3),
                burst_impulse_base: Float(0.5),
            }),
        ),
        (
            10,
            Particle::GeyserPlume(GeyserPlumeParticleData {
                water_blocks: Int(4),
            }),
        ),
        (
            15,
            Particle::DragonBreath(DragonBreathParticleData { power: Float(0.75) }),
        ),
        (
            21,
            Particle::Dust(DustParticleData {
                color: Int(0x12_34_56),
                scale: Float(1.0),
            }),
        ),
        (
            22,
            Particle::DustColorTransition(DustColorTransitionParticleData {
                from_color: Int(0x11_22_33),
                to_color: Int(0x44_55_66),
                scale: Float(2.0),
            }),
        ),
        (
            23,
            Particle::Effect(EffectParticleData {
                color: Int(0x77_88_99),
                power: Float(0.5),
            }),
        ),
        (
            28,
            Particle::EntityEffect(EntityEffectParticleData {
                color: Int(0x7f_11_22_33),
            }),
        ),
        (
            36,
            Particle::FallingDust(FallingDustParticleData {
                block_state: state(),
            }),
        ),
        (
            43,
            Particle::TintedLeaves(TintedLeavesParticleData {
                color: Int(0x7f_44_55_66),
            }),
        ),
        (
            45,
            Particle::SculkCharge(SculkChargeParticleData { roll: Float(1.25) }),
        ),
        (
            49,
            Particle::Flash(FlashParticleData {
                color: Int(0x7f_77_88_99),
            }),
        ),
        (
            53,
            Particle::InstantEffect(InstantEffectParticleData {
                color: Int(0x12_34_56),
                power: Float(0.75),
            }),
        ),
        (54, Particle::Item(ItemParticleData { item: Slot::Empty })),
        (
            55,
            Particle::Vibration(VibrationParticleData {
                source: VibrationSource::Block(BlockVibrationSource {
                    position: Position { x: 1, y: 2, z: 3 },
                }),
                ticks: VarInt(20),
            }),
        ),
        (
            56,
            Particle::Trail(TrailParticleData {
                target_x: Double(1.0),
                target_y: Double(2.0),
                target_z: Double(3.0),
                color: Int(0x12_34_56),
                duration: VarInt(40),
            }),
        ),
        (
            112,
            Particle::Shriek(ShriekParticleData { delay: VarInt(5) }),
        ),
        (
            118,
            Particle::DustPillar(DustPillarParticleData {
                block_state: state(),
            }),
        ),
        (
            122,
            Particle::BlockCrumble(BlockCrumbleParticleData {
                block_state: state(),
            }),
        ),
    ];

    assert_eq!(particles.len(), 22);
    for (expected_id, particle) in particles {
        assert_eq!(particle.type_id(), expected_id);
        assert!(!particle.is_unit());
        assert_roundtrip(particle, &[expected_id as u8]);
    }

    for (expected_id, particle) in [
        (0, Particle::AngryVillager),
        (3, Particle::Bubble),
        (111, Particle::Scrape),
        (113, Particle::EggCrack),
        (124, Particle::SulfurCubeGoo),
    ] {
        assert_eq!(particle.type_id(), expected_id);
        assert!(particle.is_unit());
        assert_roundtrip(particle, &[expected_id as u8]);
    }
}

#[test]
fn vibration_entity_source_has_its_own_typed_layout() {
    let particle = Particle::Vibration(VibrationParticleData {
        source: VibrationSource::Entity(EntityVibrationSource {
            entity_id: VarInt(300),
            eye_height: Float(1.625),
        }),
        ticks: VarInt(12),
    });
    let mut encoded = Vec::new();
    particle.encode(&mut encoded).unwrap();
    assert_eq!(encoded[0], 55);
    assert_eq!(encoded[1], 1);
    assert_eq!(&encoded[2..4], [0xac, 0x02]);
    assert_eq!(Particle::decode(&mut encoded.as_slice()).unwrap(), particle);
}

#[test]
fn optional_special_encodings_preserve_their_protocol_rules() {
    assert_roundtrip(OptionalBlockState::Absent, &[0]);
    assert_roundtrip(
        OptionalBlockState::Present(NonAirBlockStateId::new(5).unwrap()),
        &[5],
    );
    assert_eq!(
        NonAirBlockStateId::new(0).unwrap_err(),
        InvalidNonAirBlockStateId::Air
    );

    assert_roundtrip(OptionalVarInt::none(), &[0]);
    assert_roundtrip(OptionalVarInt::some(0).unwrap(), &[1]);
    assert_roundtrip(OptionalVarInt::some(41).unwrap(), &[42]);
    assert!(OptionalVarInt::some(-1).is_err());
    assert!(OptionalVarInt::some(i32::MAX).is_err());
}

#[test]
fn unknown_type_ids_are_rejected() {
    let metadata_error = EntityMetadataValue::decode(&mut [43].as_slice()).unwrap_err();
    assert_eq!(
        metadata_error.context(),
        Some(CodecKind::EntityMetadataValue)
    );
    assert_eq!(
        metadata_error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 43 })
    );

    let particle_error = Particle::decode(&mut [125].as_slice()).unwrap_err();
    assert_eq!(particle_error.codec(), CodecKind::Particle);
    assert_eq!(
        particle_error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 125 })
    );
}
