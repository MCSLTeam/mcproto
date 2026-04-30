use derive_more::with_trait::{Into, Deref, DerefMut, From};
use crate::{Codec, TypeCodecError};

const MAX_TEXT_COMPONENT_DECODE_CHARS: usize = 262_144;
const MAX_TEXT_COMPONENT_ENCODE_CHARS: usize = 32_767;


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
