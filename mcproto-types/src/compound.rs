use derive_more::with_trait::{Into, Deref, DerefMut, From};
use crate::{Codec, TypeCodecError};
use uuid::Uuid;

const MAX_TEXT_COMPONENT_DECODE_CHARS: usize = 262_144;
const MAX_TEXT_COMPONENT_ENCODE_CHARS: usize = 32_767;
const POSITION_XZ_MIN: i32 = -33_554_432;
const POSITION_XZ_MAX: i32 = 33_554_431;
const POSITION_Y_MIN: i32 = -2_048;
const POSITION_Y_MAX: i32 = 2_047;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct Angle(pub u8);

impl Angle {
    /// 从角度值创建 Angle (0.0 - 360.0)
    pub fn from_degrees(degrees: f32) -> Self {
        let normalized = degrees.rem_euclid(360.0);
        let steps = (normalized / 360.0 * 256.0).round() as u8;
        Angle(steps)
    }

    /// 转换为角度值 (0.0 - 360.0)
    pub fn to_degrees(self) -> f32 {
        (self.0 as f32) / 256.0 * 360.0
    }

    /// 从弧度创建 Angle
    pub fn from_radians(radians: f32) -> Self {
        Self::from_degrees(radians.to_degrees())
    }

    /// 转换为弧度
    pub fn to_radians(self) -> f32 {
        self.to_degrees().to_radians()
    }
}

impl Codec for Angle {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.push(self.0);
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        if buf.is_empty() {
            return Err(TypeCodecError::EmptyBuffer);
        }
        let byte = buf[0];
        *buf = &buf[1..];
        Ok(Angle(byte))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct TextComponent(pub String);

impl Codec for TextComponent {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.0.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        String::decode(buf).map(TextComponent)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct JsonTextComponent(pub String);

impl Codec for JsonTextComponent {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        let char_count = self.0.chars().count();
        if char_count > MAX_TEXT_COMPONENT_ENCODE_CHARS {
            return Err(TypeCodecError::InvalidTextComponentLength(char_count));
        }
        self.0.encode(buf)
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let text = String::decode(buf)?;
        let char_count = text.chars().count();
        if char_count > MAX_TEXT_COMPONENT_DECODE_CHARS {
            return Err(TypeCodecError::InvalidTextComponentLength(char_count));
        }
        Ok(JsonTextComponent(text))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Result<Self, TypeCodecError> {
        if !(POSITION_XZ_MIN..=POSITION_XZ_MAX).contains(&x) {
            return Err(TypeCodecError::InvalidPositionValue("x", x));
        }
        if !(POSITION_Y_MIN..=POSITION_Y_MAX).contains(&y) {
            return Err(TypeCodecError::InvalidPositionValue("y", y));
        }
        if !(POSITION_XZ_MIN..=POSITION_XZ_MAX).contains(&z) {
            return Err(TypeCodecError::InvalidPositionValue("z", z));
        }
        Ok(Self { x, y, z })
    }

    const fn decode_signed(value: u64, bits: u32) -> i32 {
        let shift = 64 - bits;
        ((value << shift) as i64 >> shift) as i32
    }
}

impl Codec for Position {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        Self::new(self.x, self.y, self.z)?;

        let x = (self.x as i64 & 0x3ff_ffff) as u64;
        let z = (self.z as i64 & 0x3ff_ffff) as u64;
        let y = (self.y as i64 & 0xfff) as u64;
        let packed = (x << 38) | (z << 12) | y;

        buf.extend_from_slice(&packed.to_be_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 8] = buf
            .get(..8)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 8))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 8))?;
        *buf = &buf[8..];

        let packed = u64::from_be_bytes(bytes);
        let x = Self::decode_signed(packed >> 38, 26);
        let z = Self::decode_signed((packed >> 12) & 0x3ff_ffff, 26);
        let y = Self::decode_signed(packed & 0xfff, 12);

        Self::new(x, y, z)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct UUID(pub Uuid);

impl Codec for UUID {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(self.0.as_bytes());
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; 16] = buf
            .get(..16)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 16))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 16))?;
        *buf = &buf[16..];

        Ok(UUID(Uuid::from_bytes(bytes)))
    }
}
