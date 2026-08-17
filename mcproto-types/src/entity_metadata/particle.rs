//! Type-safe particle definitions used by entity metadata.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind, InvalidEncodingReason};

use crate::{
    Double, Float, Int, Position, PrefixedArray, Slot, TypeCodec, TypeStructCodec, VarInt,
};

use super::BlockStateId;

macro_rules! particle_struct {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, TypeStructCodec)]
        #[type_struct_codec(kind = Particle)]
        pub struct $name {
            $($(#[$field_meta])* pub $field: $ty,)*
        }
    };
}

particle_struct!(/// Payload of `minecraft:block`.
    BlockParticleData { block_state: BlockStateId });
particle_struct!(/// Payload of `minecraft:block_marker`.
    BlockMarkerParticleData { block_state: BlockStateId });
particle_struct!(/// Payload of `minecraft:geyser`.
    GeyserParticleData { water_blocks: Int });
particle_struct!(/// Payload of `minecraft:geyser_base`.
GeyserBaseParticleData {
    water_blocks: Int,
    burst_impulse_base: Float,
});
particle_struct!(/// Payload of `minecraft:geyser_poof`.
GeyserPoofParticleData {
    water_blocks: Int,
    burst_impulse_base: Float,
});
particle_struct!(/// Payload of `minecraft:geyser_plume`.
    GeyserPlumeParticleData { water_blocks: Int });
particle_struct!(/// Payload of `minecraft:dragon_breath`.
    DragonBreathParticleData { power: Float });
particle_struct!(/// Payload of `minecraft:dust`.
DustParticleData {
    /// RGB color encoded as `0xRRGGBB`; upper bits are ignored.
    color: Int,
    /// Display scale, clamped by the client to `0.01..=4.0`.
    scale: Float,
});
particle_struct!(/// Payload of `minecraft:dust_color_transition`.
DustColorTransitionParticleData {
    /// Initial RGB color encoded as `0xRRGGBB`.
    from_color: Int,
    /// Final RGB color encoded as `0xRRGGBB`.
    to_color: Int,
    /// Display scale, clamped by the client to `0.01..=4.0`.
    scale: Float,
});
particle_struct!(/// Payload of `minecraft:effect`.
EffectParticleData {
    /// RGB color encoded as `0xRRGGBB`.
    color: Int,
    power: Float,
});
particle_struct!(/// Payload of `minecraft:entity_effect`.
    EntityEffectParticleData { color: Int });
particle_struct!(/// Payload of `minecraft:falling_dust`.
    FallingDustParticleData { block_state: BlockStateId });
particle_struct!(/// Payload of `minecraft:tinted_leaves`.
    TintedLeavesParticleData { color: Int });
particle_struct!(/// Payload of `minecraft:sculk_charge`.
    SculkChargeParticleData { roll: Float });
particle_struct!(/// Payload of `minecraft:flash`.
    FlashParticleData { color: Int });
particle_struct!(/// Payload of `minecraft:instant_effect`.
InstantEffectParticleData {
    /// RGB color encoded as `0xRRGGBB`.
    color: Int,
    power: Float,
});
particle_struct!(/// Payload of `minecraft:item`.
    ItemParticleData { item: Slot });
particle_struct!(/// Payload of `minecraft:trail`.
TrailParticleData {
    target_x: Double,
    target_y: Double,
    target_z: Double,
    /// Trail RGB color encoded as `0xRRGGBB`.
    color: Int,
    /// Lifetime in ticks.
    duration: VarInt,
});
particle_struct!(/// Payload of `minecraft:shriek`.
    ShriekParticleData { delay: VarInt });
particle_struct!(/// Payload of `minecraft:dust_pillar`.
    DustPillarParticleData { block_state: BlockStateId });
particle_struct!(/// Payload of `minecraft:block_crumble`.
    BlockCrumbleParticleData { block_state: BlockStateId });

/// Block position payload of the `minecraft:block` vibration source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeStructCodec)]
#[type_struct_codec(kind = VibrationSource)]
pub struct BlockVibrationSource {
    /// Position from which the vibration originated.
    pub position: Position,
}

/// Entity payload of the `minecraft:entity` vibration source.
#[derive(Debug, Clone, Copy, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = VibrationSource)]
pub struct EntityVibrationSource {
    /// Runtime entity ID from which the vibration originated.
    pub entity_id: VarInt,
    /// Eye height relative to the entity.
    pub eye_height: Float,
}

/// Position source selected by a vibration particle's registry type ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VibrationSource {
    /// Type ID 0, `minecraft:block`.
    Block(BlockVibrationSource),
    /// Type ID 1, `minecraft:entity`.
    Entity(EntityVibrationSource),
}

impl TypeCodec for VibrationSource {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        match self {
            Self::Block(value) => {
                encode_vibration_source_type(0, writer)?;
                value.encode(writer)
            }
            Self::Entity(value) => {
                encode_vibration_source_type(1, writer)?;
                value.encode(writer)
            }
        }
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let source_type = VarInt::decode(reader)
            .map_err(|error| error.with_context(CodecKind::VibrationSource))?
            .0;
        match source_type {
            0 => BlockVibrationSource::decode(reader).map(Self::Block),
            1 => EntityVibrationSource::decode(reader).map(Self::Entity),
            value => Err(CodecError::invalid_encoding(
                CodecKind::VibrationSource,
                0,
                InvalidEncodingReason::InvalidEnumValue {
                    value: i128::from(value),
                },
            )),
        }
    }
}

fn encode_vibration_source_type(
    source_type: i32,
    writer: &mut impl Write,
) -> Result<(), CodecError> {
    VarInt(source_type)
        .encode(writer)
        .map_err(|error| error.with_context(CodecKind::VibrationSource))
}

particle_struct!(/// Payload of `minecraft:vibration`.
VibrationParticleData {
    /// Type-safe block or entity position source.
    source: VibrationSource,
    /// Travel time in ticks.
    ticks: VarInt,
});

macro_rules! define_particles {
    (
        unit {
            $($unit_id:literal => $unit_variant:ident = $unit_name:literal,)*
        }
        data {
            $($data_id:literal => $data_variant:ident($data_type:ty) = $data_name:literal,)*
        }
    ) => {
        /// A particle type ID bound to exactly the payload required by that type.
        ///
        /// Unit variants encode only their registry ID. Data variants encode the
        /// registry ID followed by their dedicated payload structure, preventing
        /// particle IDs and payload layouts from being mismatched in memory.
        ///
        /// See the official [particle protocol table].
        ///
        /// [particle protocol table]: https://minecraft.wiki/w/Java_Edition_protocol/Particles
        #[derive(Debug, Clone, PartialEq)]
        pub enum Particle {
            $(
                #[doc = concat!("The `", $unit_name, "` particle (type ID `", stringify!($unit_id), "`) with no payload.")]
                $unit_variant,
            )*
            $(
                #[doc = concat!("The `", $data_name, "` particle (type ID `", stringify!($data_id), "`).")]
                $data_variant($data_type),
            )*
        }

        impl Particle {
            /// Returns the numeric ID in the `minecraft:particle_type` registry.
            #[must_use]
            pub const fn type_id(&self) -> i32 {
                match self {
                    $(Self::$unit_variant => $unit_id,)*
                    $(Self::$data_variant(_) => $data_id,)*
                }
            }

            /// Returns the namespaced particle registry name.
            #[must_use]
            pub const fn registry_name(&self) -> &'static str {
                match self {
                    $(Self::$unit_variant => $unit_name,)*
                    $(Self::$data_variant(_) => $data_name,)*
                }
            }

            /// Returns whether this particle has no type-specific payload.
            #[must_use]
            pub const fn is_unit(&self) -> bool {
                match self {
                    $(Self::$unit_variant => true,)*
                    $(Self::$data_variant(_) => false,)*
                }
            }
        }

        impl TypeCodec for Particle {
            fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
                match self {
                    $(Self::$unit_variant => encode_particle_type($unit_id, writer),)*
                    $(
                        Self::$data_variant(value) => {
                            encode_particle_type($data_id, writer)?;
                            value.encode(writer)
                        }
                    )*
                }
            }

            fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
                let particle_type = VarInt::decode(reader)
                    .map_err(|error| error.with_context(CodecKind::Particle))?
                    .0;
                match particle_type {
                    $($unit_id => Ok(Self::$unit_variant),)*
                    $($data_id => <$data_type>::decode(reader).map(Self::$data_variant),)*
                    value => Err(CodecError::invalid_encoding(
                        CodecKind::Particle,
                        0,
                        InvalidEncodingReason::InvalidEnumValue {
                            value: i128::from(value),
                        },
                    )),
                }
            }
        }
    };
}

define_particles! {
    unit {
        0 => AngryVillager = "minecraft:angry_villager",
        3 => Bubble = "minecraft:bubble",
        4 => SulfurBubbles = "minecraft:sulfur_bubbles",
        5 => NoxiousGas = "minecraft:noxious_gas",
        6 => NoxiousGasCloud = "minecraft:noxious_gas_cloud",
        11 => Cloud = "minecraft:cloud",
        12 => CopperFireFlame = "minecraft:copper_fire_flame",
        13 => Crit = "minecraft:crit",
        14 => DamageIndicator = "minecraft:damage_indicator",
        16 => DrippingLava = "minecraft:dripping_lava",
        17 => FallingLava = "minecraft:falling_lava",
        18 => LandingLava = "minecraft:landing_lava",
        19 => DrippingWater = "minecraft:dripping_water",
        20 => FallingWater = "minecraft:falling_water",
        24 => ElderGuardian = "minecraft:elder_guardian",
        25 => EnchantedHit = "minecraft:enchanted_hit",
        26 => Enchant = "minecraft:enchant",
        27 => EndRod = "minecraft:end_rod",
        29 => ExplosionEmitter = "minecraft:explosion_emitter",
        30 => Explosion = "minecraft:explosion",
        31 => Gust = "minecraft:gust",
        32 => SmallGust = "minecraft:small_gust",
        33 => GustEmitterLarge = "minecraft:gust_emitter_large",
        34 => GustEmitterSmall = "minecraft:gust_emitter_small",
        35 => SonicBoom = "minecraft:sonic_boom",
        37 => Firework = "minecraft:firework",
        38 => Fishing = "minecraft:fishing",
        39 => Flame = "minecraft:flame",
        40 => Infested = "minecraft:infested",
        41 => CherryLeaves = "minecraft:cherry_leaves",
        42 => PaleOakLeaves = "minecraft:pale_oak_leaves",
        44 => SculkSoul = "minecraft:sculk_soul",
        46 => SculkChargePop = "minecraft:sculk_charge_pop",
        47 => SoulFireFlame = "minecraft:soul_fire_flame",
        48 => Soul = "minecraft:soul",
        50 => HappyVillager = "minecraft:happy_villager",
        51 => Composter = "minecraft:composter",
        52 => Heart = "minecraft:heart",
        57 => PauseMobGrowth = "minecraft:pause_mob_growth",
        58 => ResetMobGrowth = "minecraft:reset_mob_growth",
        59 => ItemSlime = "minecraft:item_slime",
        60 => ItemCobweb = "minecraft:item_cobweb",
        61 => ItemSnowball = "minecraft:item_snowball",
        62 => LargeSmoke = "minecraft:large_smoke",
        63 => Lava = "minecraft:lava",
        64 => Mycelium = "minecraft:mycelium",
        65 => Note = "minecraft:note",
        66 => Poof = "minecraft:poof",
        67 => Portal = "minecraft:portal",
        68 => Rain = "minecraft:rain",
        69 => Smoke = "minecraft:smoke",
        70 => WhiteSmoke = "minecraft:white_smoke",
        71 => Sneeze = "minecraft:sneeze",
        72 => Spit = "minecraft:spit",
        73 => SquidInk = "minecraft:squid_ink",
        74 => SweepAttack = "minecraft:sweep_attack",
        75 => TotemOfUndying = "minecraft:totem_of_undying",
        76 => Underwater = "minecraft:underwater",
        77 => Splash = "minecraft:splash",
        78 => Witch = "minecraft:witch",
        79 => BubblePop = "minecraft:bubble_pop",
        80 => CurrentDown = "minecraft:current_down",
        81 => BubbleColumnUp = "minecraft:bubble_column_up",
        82 => Nautilus = "minecraft:nautilus",
        83 => Dolphin = "minecraft:dolphin",
        84 => CampfireCosySmoke = "minecraft:campfire_cosy_smoke",
        85 => CampfireSignalSmoke = "minecraft:campfire_signal_smoke",
        86 => DrippingHoney = "minecraft:dripping_honey",
        87 => FallingHoney = "minecraft:falling_honey",
        88 => LandingHoney = "minecraft:landing_honey",
        89 => FallingNectar = "minecraft:falling_nectar",
        90 => FallingSporeBlossom = "minecraft:falling_spore_blossom",
        91 => Ash = "minecraft:ash",
        92 => CrimsonSpore = "minecraft:crimson_spore",
        93 => WarpedSpore = "minecraft:warped_spore",
        94 => SporeBlossomAir = "minecraft:spore_blossom_air",
        95 => DrippingObsidianTear = "minecraft:dripping_obsidian_tear",
        96 => FallingObsidianTear = "minecraft:falling_obsidian_tear",
        97 => LandingObsidianTear = "minecraft:landing_obsidian_tear",
        98 => ReversePortal = "minecraft:reverse_portal",
        99 => WhiteAsh = "minecraft:white_ash",
        100 => SmallFlame = "minecraft:small_flame",
        101 => Snowflake = "minecraft:snowflake",
        102 => DrippingDripstoneLava = "minecraft:dripping_dripstone_lava",
        103 => FallingDripstoneLava = "minecraft:falling_dripstone_lava",
        104 => DrippingDripstoneWater = "minecraft:dripping_dripstone_water",
        105 => FallingDripstoneWater = "minecraft:falling_dripstone_water",
        106 => GlowSquidInk = "minecraft:glow_squid_ink",
        107 => Glow = "minecraft:glow",
        108 => WaxOn = "minecraft:wax_on",
        109 => WaxOff = "minecraft:wax_off",
        110 => ElectricSpark = "minecraft:electric_spark",
        111 => Scrape = "minecraft:scrape",
        113 => EggCrack = "minecraft:egg_crack",
        114 => DustPlume = "minecraft:dust_plume",
        115 => TrialSpawnerDetection = "minecraft:trial_spawner_detection",
        116 => TrialSpawnerDetectionOminous = "minecraft:trial_spawner_detection_ominous",
        117 => VaultConnection = "minecraft:vault_connection",
        119 => OminousSpawning = "minecraft:ominous_spawning",
        120 => RaidOmen = "minecraft:raid_omen",
        121 => TrialOmen = "minecraft:trial_omen",
        123 => Firefly = "minecraft:firefly",
        124 => SulfurCubeGoo = "minecraft:sulfur_cube_goo",
    }
    data {
        1 => Block(BlockParticleData) = "minecraft:block",
        2 => BlockMarker(BlockMarkerParticleData) = "minecraft:block_marker",
        7 => Geyser(GeyserParticleData) = "minecraft:geyser",
        8 => GeyserBase(GeyserBaseParticleData) = "minecraft:geyser_base",
        9 => GeyserPoof(GeyserPoofParticleData) = "minecraft:geyser_poof",
        10 => GeyserPlume(GeyserPlumeParticleData) = "minecraft:geyser_plume",
        15 => DragonBreath(DragonBreathParticleData) = "minecraft:dragon_breath",
        21 => Dust(DustParticleData) = "minecraft:dust",
        22 => DustColorTransition(DustColorTransitionParticleData) = "minecraft:dust_color_transition",
        23 => Effect(EffectParticleData) = "minecraft:effect",
        28 => EntityEffect(EntityEffectParticleData) = "minecraft:entity_effect",
        36 => FallingDust(FallingDustParticleData) = "minecraft:falling_dust",
        43 => TintedLeaves(TintedLeavesParticleData) = "minecraft:tinted_leaves",
        45 => SculkCharge(SculkChargeParticleData) = "minecraft:sculk_charge",
        49 => Flash(FlashParticleData) = "minecraft:flash",
        53 => InstantEffect(InstantEffectParticleData) = "minecraft:instant_effect",
        54 => Item(ItemParticleData) = "minecraft:item",
        55 => Vibration(VibrationParticleData) = "minecraft:vibration",
        56 => Trail(TrailParticleData) = "minecraft:trail",
        112 => Shriek(ShriekParticleData) = "minecraft:shriek",
        118 => DustPillar(DustPillarParticleData) = "minecraft:dust_pillar",
        122 => BlockCrumble(BlockCrumbleParticleData) = "minecraft:block_crumble",
    }
}

fn encode_particle_type(particle_type: i32, writer: &mut impl Write) -> Result<(), CodecError> {
    VarInt(particle_type)
        .encode(writer)
        .map_err(|error| error.with_context(CodecKind::Particle))
}

/// A VarInt-length-prefixed list of complete particle definitions.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, TypeStructCodec)]
#[type_struct_codec(kind = EntityMetadataValue)]
pub struct Particles(pub PrefixedArray<Particle>);

impl Particles {
    /// Creates a particle list.
    #[must_use]
    pub const fn new(values: Vec<Particle>) -> Self {
        Self(PrefixedArray(values))
    }

    /// Returns the particle definitions.
    #[must_use]
    pub fn as_slice(&self) -> &[Particle] {
        self.0.as_slice()
    }

    /// Returns the number of particle definitions.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the list is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Extracts the particle definitions.
    #[must_use]
    pub fn into_vec(self) -> Vec<Particle> {
        self.0.into_vec()
    }
}

impl From<Vec<Particle>> for Particles {
    fn from(values: Vec<Particle>) -> Self {
        Self::new(values)
    }
}

impl From<Particles> for Vec<Particle> {
    fn from(values: Particles) -> Self {
        values.into_vec()
    }
}

impl Default for Particles {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
