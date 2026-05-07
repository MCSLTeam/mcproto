pub mod component;
pub mod enums;
pub mod subtypes;

use crate::basic::VarInt;
use crate::{Codec, TypeCodecError};
use derive_more::with_trait::{Deref, DerefMut, From, Into};
use mcproto_codec::{VarIntRead, VarIntWrite};
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

// LpVec3
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LpVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Bit Set
#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct BitSet(pub Vec<i64>);

impl BitSet {
    pub fn new(data: Vec<i64>) -> Self {
        Self(data)
    }

    pub fn get(&self, index: usize) -> bool {
        let long_index = index / 64;
        let bit_index = index % 64;
        self.0
            .get(long_index)
            .map(|value| ((*value as u64) & (1u64 << bit_index)) != 0)
            .unwrap_or(false)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let long_index = index / 64;
        let bit_index = index % 64;
        if long_index >= self.0.len() {
            self.0.resize(long_index + 1, 0);
        }
        let mut bits = self.0[long_index] as u64;
        let mask = 1u64 << bit_index;
        if value {
            bits |= mask;
        } else {
            bits &= !mask;
        }
        self.0[long_index] = bits as i64;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, Into, Deref, DerefMut)]
pub struct FixedBitSet<const BYTES: usize>(pub [u8; BYTES]);

impl<const BYTES: usize> FixedBitSet<BYTES> {
    pub const fn new(data: [u8; BYTES]) -> Self {
        Self(data)
    }

    pub const fn byte_len() -> usize {
        BYTES
    }

    pub const fn bit_capacity() -> usize {
        BYTES * 8
    }

    pub fn get(&self, index: usize) -> bool {
        if index >= Self::bit_capacity() {
            return false;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.0[byte_index] & (1 << bit_index)) != 0
    }

    pub fn set(&mut self, index: usize, value: bool) {
        if index >= Self::bit_capacity() {
            return;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        let mask = 1u8 << bit_index;
        if value {
            self.0[byte_index] |= mask;
        } else {
            self.0[byte_index] &= !mask;
        }
    }
}

impl<const BYTES: usize> Codec for FixedBitSet<BYTES> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0);
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let bytes: [u8; BYTES] = buf
            .get(..BYTES)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), BYTES))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), BYTES))?;
        *buf = &buf[BYTES..];
        Ok(Self(bytes))
    }
}

impl Codec for BitSet {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        let len = i32::try_from(self.0.len())
            .map_err(|_| TypeCodecError::InvalidArrayLength(i32::MAX))?;
        buf.write_varint(len)?;
        for value in &self.0 {
            buf.extend_from_slice(&value.to_be_bytes());
        }
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let len = buf.read_varint()?;
        if len < 0 {
            return Err(TypeCodecError::InvalidArrayLength(len));
        }
        let len = len as usize;
        let mut data = Vec::with_capacity(len);

        for _ in 0..len {
            let bytes: [u8; 8] = buf
                .get(..8)
                .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 8))?
                .try_into()
                .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 8))?;
            *buf = &buf[8..];
            data.push(i64::from_be_bytes(bytes));
        }

        Ok(Self(data))
    }
}

impl LpVec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Codec for LpVec3 {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        const MIN_THRESHOLD: f64 = 3.051_944_088_384_301e-5;
        const MAX_COORDINATE: f64 = 17_179_869_183.0;
        const MAX_QUANTIZED_VALUE: f64 = 32_766.0;

        let x = self.x.clamp(-MAX_COORDINATE, MAX_COORDINATE);
        let y = self.y.clamp(-MAX_COORDINATE, MAX_COORDINATE);
        let z = self.z.clamp(-MAX_COORDINATE, MAX_COORDINATE);

        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            buf.push(0);
            return Ok(());
        }

        let max_coordinate = x.abs().max(y.abs()).max(z.abs());
        if max_coordinate < MIN_THRESHOLD {
            buf.push(0);
            return Ok(());
        }

        let max_coordinate_i = max_coordinate as i64;
        let scale_factor = if max_coordinate > max_coordinate_i as f64 {
            max_coordinate_i + 1
        } else {
            max_coordinate_i
        };

        let need_continuation = (scale_factor & 3) != scale_factor;
        let packed_scale = if need_continuation {
            (scale_factor & 3) | 4
        } else {
            scale_factor
        };

        let pack = |v: f64| -> u64 {
            let normalized = v / scale_factor as f64;
            let quantized = ((normalized * 0.5 + 0.5) * MAX_QUANTIZED_VALUE).round();
            (quantized as u64) & 0x7fff
        };

        let packed_x = pack(x) << 3;
        let packed_y = pack(y) << 18;
        let packed_z = pack(z) << 33;
        let packed = packed_z | packed_y | packed_x | (packed_scale as u64);

        buf.push(packed as u8);
        buf.push((packed >> 8) as u8);
        buf.extend_from_slice(&((packed >> 16) as u32).to_be_bytes());
        if need_continuation {
            buf.write_varint((scale_factor >> 2) as i32)?;
        }
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        const MAX_QUANTIZED_VALUE: f64 = 32_766.0;

        let byte1 = *buf
            .first()
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 1))?;
        *buf = &buf[1..];

        if byte1 == 0 {
            return Ok(Self::new(0.0, 0.0, 0.0));
        }

        let byte2 = *buf
            .first()
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 1))?;
        *buf = &buf[1..];

        let bytes3_to_6: [u8; 4] = buf
            .get(..4)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), 4))?
            .try_into()
            .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), 4))?;
        *buf = &buf[4..];

        let bytes3_to_6 = u32::from_be_bytes(bytes3_to_6) as u64;
        let packed = (bytes3_to_6 << 16) | ((byte2 as u64) << 8) | (byte1 as u64);

        let mut scale_factor = (byte1 & 3) as u64;
        if (byte1 & 4) == 4 {
            scale_factor |= ((buf.read_varint()? as u32 as u64) << 2) & 0xffff_ffff_ffff_fffc;
        }

        let unpack = |v: u64| -> f64 {
            let q = (v & 0x7fff).min(32_766) as f64;
            q * 2.0 / MAX_QUANTIZED_VALUE - 1.0
        };

        let sf = scale_factor as f64;
        Ok(Self {
            x: unpack(packed >> 3) * sf,
            y: unpack(packed >> 18) * sf,
            z: unpack(packed >> 33) * sf,
        })
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
// TODO: EntityMetadata
// TODO: Slot
// TODO: Hashed Slot

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nbt(Vec<u8>);
impl Codec for Nbt {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        buf.extend_from_slice(&self.0);
        Ok(())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        let start = *buf;
        let mut pos = 0usize;
        skip_named_tag(start, &mut pos)?;

        *buf = &start[pos..];
        Ok(Nbt(start[..pos].to_vec()))
    }
}

fn read_u8(buf: &[u8], pos: &mut usize) -> Result<u8, TypeCodecError> {
    let b = *buf
        .get(*pos)
        .ok_or(TypeCodecError::EndOfBuffer(*pos, *pos + 1))?;
    *pos += 1;
    Ok(b)
}

fn skip_bytes(buf: &[u8], pos: &mut usize, n: usize) -> Result<(), TypeCodecError> {
    let end = pos
        .checked_add(n)
        .ok_or(TypeCodecError::EndOfBuffer(*pos, usize::MAX))?;
    if end > buf.len() {
        return Err(TypeCodecError::EndOfBuffer(buf.len(), end));
    }
    *pos = end;
    Ok(())
}

fn read_i32_be(buf: &[u8], pos: &mut usize) -> Result<i32, TypeCodecError> {
    let end = pos
        .checked_add(4)
        .ok_or(TypeCodecError::EndOfBuffer(*pos, usize::MAX))?;
    let bytes: [u8; 4] = buf
        .get(*pos..end)
        .ok_or(TypeCodecError::EndOfBuffer(buf.len(), end))?
        .try_into()
        .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), end))?;
    *pos = end;
    Ok(i32::from_be_bytes(bytes))
}

fn read_u16_be(buf: &[u8], pos: &mut usize) -> Result<u16, TypeCodecError> {
    let end = pos
        .checked_add(2)
        .ok_or(TypeCodecError::EndOfBuffer(*pos, usize::MAX))?;
    let bytes: [u8; 2] = buf
        .get(*pos..end)
        .ok_or(TypeCodecError::EndOfBuffer(buf.len(), end))?
        .try_into()
        .map_err(|_| TypeCodecError::EndOfBuffer(buf.len(), end))?;
    *pos = end;
    Ok(u16::from_be_bytes(bytes))
}

fn skip_string(buf: &[u8], pos: &mut usize) -> Result<(), TypeCodecError> {
    let len = read_u16_be(buf, pos)? as usize;
    skip_bytes(buf, pos, len)
}

fn skip_named_tag(buf: &[u8], pos: &mut usize) -> Result<(), TypeCodecError> {
    let tag_id = read_u8(buf, pos)?;
    if tag_id == 0x00 {
        return Err(TypeCodecError::UnknownNbtTag(tag_id));
    }
    skip_string(buf, pos)?; // name
    skip_tag_payload(buf, pos, tag_id)
}

fn skip_tag_payload(buf: &[u8], pos: &mut usize, tag_id: u8) -> Result<(), TypeCodecError> {
    match tag_id {
        0x00 => Ok(()), // End (仅用于 Compound 内部)
        0x01 => skip_bytes(buf, pos, 1), // Byte
        0x02 => skip_bytes(buf, pos, 2), // Short
        0x03 => skip_bytes(buf, pos, 4), // Int
        0x04 => skip_bytes(buf, pos, 8), // Long
        0x05 => skip_bytes(buf, pos, 4), // Float
        0x06 => skip_bytes(buf, pos, 8), // Double
        0x07 => {
            let len = read_i32_be(buf, pos)?;
            if len < 0 {
                return Err(TypeCodecError::InvalidArrayLength(len));
            }
            skip_bytes(buf, pos, len as usize)
        }
        0x08 => skip_string(buf, pos), // String
        0x09 => {
            let elem_tag = read_u8(buf, pos)?;
            let len = read_i32_be(buf, pos)?;
            if len < 0 {
                return Err(TypeCodecError::InvalidArrayLength(len));
            }
            for _ in 0..(len as usize) {
                skip_tag_payload(buf, pos, elem_tag)?;
            }
            Ok(())
        }
        0x0A => {
            loop {
                let inner_tag = read_u8(buf, pos)?;
                if inner_tag == 0x00 {
                    break; // TAG_End
                }
                skip_string(buf, pos)?; // field name
                skip_tag_payload(buf, pos, inner_tag)?;
            }
            Ok(())
        }
        0x0B => {
            let len = read_i32_be(buf, pos)?;
            if len < 0 {
                return Err(TypeCodecError::InvalidArrayLength(len));
            }
            let bytes_len = (len as usize)
                .checked_mul(4)
                .ok_or(TypeCodecError::InvalidArrayLength(i32::MAX))?;
            skip_bytes(buf, pos, bytes_len)
        }
        0x0C => {
            let len = read_i32_be(buf, pos)?;
            if len < 0 {
                return Err(TypeCodecError::InvalidArrayLength(len));
            }
            let bytes_len = (len as usize)
                .checked_mul(8)
                .ok_or(TypeCodecError::InvalidArrayLength(i32::MAX))?;
            skip_bytes(buf, pos, bytes_len)
        }
        _ => Err(TypeCodecError::UnknownNbtTag(tag_id)),
    }
}
