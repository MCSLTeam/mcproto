//! Boss Bar packet for protocol 776 (Minecraft Java Edition 26.2).

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind};
use mcproto_types::{Float, ProtocolEnum, TextComponent, TypeCodec, UnsignedByte, Uuid, VarInt};

use crate::PacketCodec;

/// Color of a boss bar.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Boss_Bar)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum BossBarColor {
    /// Pink.
    Pink = 0,
    /// Blue.
    Blue = 1,
    /// Red.
    Red = 2,
    /// Green.
    Green = 3,
    /// Yellow.
    Yellow = 4,
    /// Purple.
    Purple = 5,
    /// White.
    White = 6,
}

/// Type of division for a boss bar.
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Boss_Bar)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum BossBarDivision {
    /// No division.
    NoDivision = 0,
    /// 6 notches.
    SixNotches = 1,
    /// 10 notches.
    TenNotches = 2,
    /// 12 notches.
    TwelveNotches = 3,
    /// 20 notches.
    TwentyNotches = 4,
}

/// Bit mask of boss bar flags.
///
/// 0x01: should darken sky, 0x02: is dragon bar (used to play end music),
/// 0x04: create fog (previously was also controlled by 0x02).
///
/// Unknown bits are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BossBarFlags(pub u8);

impl BossBarFlags {
    /// Should darken sky.
    pub const DARKEN_SKY: Self = Self(0x01);
    /// Is dragon bar (used to play end music).
    pub const DRAGON_BAR: Self = Self(0x02);
    /// Create fog (previously was also controlled by 0x02).
    pub const CREATE_FOG: Self = Self(0x04);

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

impl TypeCodec for BossBarFlags {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        UnsignedByte(self.0).encode(writer)
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        UnsignedByte::decode(reader).map(|value| Self(value.0))
    }
}

/// Boss bar action discriminator.
///
/// Determines the layout of the remaining packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum BossBarActionType {
    /// Add.
    Add = 0,
    /// Remove.
    Remove = 1,
    /// Update health.
    UpdateHealth = 2,
    /// Update title.
    UpdateTitle = 3,
    /// Update style.
    UpdateStyle = 4,
    /// Update flags.
    UpdateFlags = 5,
}

/// A boss bar action paired with its payload.
///
/// The action discriminator is derived from the variant, so an action cannot
/// be paired with the wrong payload.
#[derive(Debug, Clone, PartialEq)]
pub enum BossBarAction {
    /// Adds a boss bar.
    Add {
        /// Title.
        title: TextComponent,
        /// From 0 to 1. Values greater than 1 do not crash a vanilla client, and start rendering part of a second health bar at around 1.5.
        health: Float,
        /// Color ID (see below).
        color: BossBarColor,
        /// Type of division (see below).
        division: BossBarDivision,
        /// Bit mask. 0x01: should darken sky, 0x02: is dragon bar (used to play end music), 0x04: create fog (previously was also controlled by 0x02).
        flags: BossBarFlags,
    },
    /// Removes this boss bar.
    Remove,
    /// Updates the health.
    UpdateHealth {
        /// as above
        health: Float,
    },
    /// Updates the title.
    UpdateTitle {
        /// Title.
        title: TextComponent,
    },
    /// Updates the style.
    UpdateStyle {
        /// Color ID (see below).
        color: BossBarColor,
        /// as above
        division: BossBarDivision,
    },
    /// Updates the flags.
    UpdateFlags {
        /// as above
        flags: BossBarFlags,
    },
}

impl BossBarAction {
    /// Returns the action discriminator for this action.
    #[must_use]
    pub const fn action_type(&self) -> BossBarActionType {
        match self {
            Self::Add { .. } => BossBarActionType::Add,
            Self::Remove => BossBarActionType::Remove,
            Self::UpdateHealth { .. } => BossBarActionType::UpdateHealth,
            Self::UpdateTitle { .. } => BossBarActionType::UpdateTitle,
            Self::UpdateStyle { .. } => BossBarActionType::UpdateStyle,
            Self::UpdateFlags { .. } => BossBarActionType::UpdateFlags,
        }
    }
}

impl TypeCodec for BossBarAction {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.action_type()
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::TypeStruct))?;

        match self {
            Self::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                title.encode(writer)?;
                health.encode(writer)?;
                color.encode(writer)?;
                division.encode(writer)?;
                flags.encode(writer)?;
            }
            Self::Remove => {}
            Self::UpdateHealth { health } => health.encode(writer)?,
            Self::UpdateTitle { title } => title.encode(writer)?,
            Self::UpdateStyle { color, division } => {
                color.encode(writer)?;
                division.encode(writer)?;
            }
            Self::UpdateFlags { flags } => flags.encode(writer)?,
        }

        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let action_type = BossBarActionType::decode(reader)
            .map_err(|error| error.with_context(CodecKind::TypeStruct))?;

        Ok(match action_type {
            BossBarActionType::Add => Self::Add {
                title: TextComponent::decode(reader)?,
                health: Float::decode(reader)?,
                color: BossBarColor::decode(reader)?,
                division: BossBarDivision::decode(reader)?,
                flags: BossBarFlags::decode(reader)?,
            },
            BossBarActionType::Remove => Self::Remove,
            BossBarActionType::UpdateHealth => Self::UpdateHealth {
                health: Float::decode(reader)?,
            },
            BossBarActionType::UpdateTitle => Self::UpdateTitle {
                title: TextComponent::decode(reader)?,
            },
            BossBarActionType::UpdateStyle => Self::UpdateStyle {
                color: BossBarColor::decode(reader)?,
                division: BossBarDivision::decode(reader)?,
            },
            BossBarActionType::UpdateFlags => Self::UpdateFlags {
                flags: BossBarFlags::decode(reader)?,
            },
        })
    }
}

#[derive(PacketCodec)]
#[packet(
    name = "boss_event",
    id = 0x09,
    state = Play,
    direction = Clientbound,
)]
/// Clientbound `boss_event`, Play ID: 9 (0x9)
///
/// [Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Boss_Bar)
pub struct BossBar {
    /// Unique ID for this bar.
    pub uuid: Uuid,
    /// Determines the layout of the remaining packet.
    pub action: BossBarAction,
}
