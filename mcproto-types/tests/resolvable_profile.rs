//! Protocol tests for Resolvable Profile.

use mcproto_codec::error::{CodecErrorKind, CodecKind, InvalidEncodingReason};
use mcproto_types::{
    GameProfile, GameProfileUsername, Identifier, PartialProfile, PrefixedOptional,
    ProfileComponent, ResolvableProfile, ResolvableProfileData, SkinModel, TypeCodec, Uuid,
};

const UUID_BYTES: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21,
];

#[test]
fn partial_profile_fields_follow_the_documented_wire_order() {
    let value = ResolvableProfile {
        profile: ResolvableProfileData::Partial(PartialProfile {
            username: PrefixedOptional::some(GameProfileUsername::new("Alex").unwrap()),
            ..PartialProfile::default()
        }),
        body: PrefixedOptional::some(Identifier::new("a:b").unwrap()),
        cape: PrefixedOptional::none(),
        elytra: PrefixedOptional::none(),
        model: PrefixedOptional::some(SkinModel::Slim),
    };

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    assert_eq!(
        encoded,
        [
            0, // Partial profile kind
            1, 4, b'A', b'l', b'e', b'x', // Username
            0,    // UUID absent
            0,    // Empty property list
            1, 3, b'a', b':', b'b', // Body override
            0,    // Cape absent
            0,    // Elytra absent
            1, 1, // Slim model
        ]
    );

    let mut input = encoded.as_slice();
    assert_eq!(ResolvableProfile::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn complete_profile_reuses_game_profile_encoding() {
    let game_profile = GameProfile::new(
        Uuid::from_bytes(UUID_BYTES),
        GameProfileUsername::new("Alex").unwrap(),
    );
    let value = ResolvableProfile::complete(game_profile.clone());

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();

    let mut expected = vec![1];
    game_profile.encode(&mut expected).unwrap();
    expected.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(ResolvableProfile::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn every_texture_override_roundtrips() {
    let value = ResolvableProfile {
        profile: ResolvableProfileData::Partial(PartialProfile::default()),
        body: PrefixedOptional::some(Identifier::new("minecraft:body").unwrap()),
        cape: PrefixedOptional::some(Identifier::new("minecraft:cape").unwrap()),
        elytra: PrefixedOptional::some(Identifier::new("minecraft:elytra").unwrap()),
        model: PrefixedOptional::some(SkinModel::Wide),
    };

    let mut encoded = Vec::new();
    value.encode(&mut encoded).unwrap();
    let mut input = encoded.as_slice();
    assert_eq!(ResolvableProfile::decode(&mut input).unwrap(), value);
    assert!(input.is_empty());
}

#[test]
fn invalid_profile_kind_is_rejected() {
    let error = ResolvableProfile::decode(&mut [2].as_slice()).unwrap_err();
    assert_eq!(error.codec(), CodecKind::ResolvableProfile);
    assert_eq!(
        error.kind(),
        CodecErrorKind::InvalidEncoding(InvalidEncodingReason::InvalidEnumValue { value: 2 })
    );
}

#[test]
fn slot_profile_component_reuses_the_canonical_type() {
    let profile = ResolvableProfile::partial(PartialProfile {
        username: PrefixedOptional::some(GameProfileUsername::new("Steve").unwrap()),
        ..PartialProfile::default()
    });
    let component = ProfileComponent {
        profile: profile.clone(),
    };

    let mut expected = Vec::new();
    profile.encode(&mut expected).unwrap();
    let mut encoded = Vec::new();
    component.encode(&mut encoded).unwrap();
    assert_eq!(encoded, expected);

    let mut input = encoded.as_slice();
    assert_eq!(ProfileComponent::decode(&mut input).unwrap(), component);
    assert!(input.is_empty());

    let _: &mcproto_types::slot::ResolvableProfile = &component.profile;
}
