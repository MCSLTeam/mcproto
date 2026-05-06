use crate::{Codec, ContextualCodec, Ctx, TypeCodecError};
use mcproto_codec::{VarIntRead, VarIntWrite};
use crate::basic::{Identifier, Float};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Optional<T>(pub Option<T>);

impl<T> Optional<T> {
    pub const fn new(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> ContextualCodec<Ctx> for Optional<T>
where
    T: Codec,
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, ctx: &Ctx) -> Result<(), TypeCodecError> {
        let present = ctx.present.ok_or(TypeCodecError::MissingContext("present"))?;
        match (present, self.0.as_ref()) {
            (true, Some(value)) => value.encode(buf),
            (false, None) => Ok(()),
            (true, None) => Err(TypeCodecError::ContextMismatch("present=true but value=None")),
            (false, Some(_)) => Err(TypeCodecError::ContextMismatch("present=false but value=Some")),
        }
    }

    fn decode_with_ctx(buf: &mut &[u8], ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let present = ctx.present.ok_or(TypeCodecError::MissingContext("present"))?;
        if present {
            Ok(Self(Some(T::decode(buf)?)))
        } else {
            Ok(Self(None))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixedOptional<T>(pub Option<T>);
impl<T> PrefixedOptional<T> {
    pub const fn new(value: Option<T>) -> Self {
        Self(value)
    }
}
impl<T: Codec> Codec for PrefixedOptional<T> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.encode_with_ctx(buf, &Ctx::none())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Self::decode_with_ctx(buf, &Ctx::none())
    }
}
impl<T> ContextualCodec<Ctx> for PrefixedOptional<T>
where T: Codec,
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, _ctx: &Ctx) -> Result<(), TypeCodecError> {
        match self.0.as_ref() {
            Some(value) => {
                true.encode(buf)?;
                value.encode(buf)
            }
            None => {
                false.encode(buf)?;
                Ok(())
            }
        }
    }

    fn decode_with_ctx(buf: &mut &[u8], _ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let present = bool::decode(buf)?;
        if present {
            Ok(Self(Some(T::decode(buf)?)))
        } else {
            Ok(Self(None))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array<T>(pub Vec<T>);
impl<T> Array<T> {
    pub const fn new(values: Vec<T>) -> Self {
        Self(values)
    }
}
impl<T> ContextualCodec<Ctx> for Array<T>
where
    T: Codec
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, ctx: &Ctx) -> Result<(), TypeCodecError> {
        let len = ctx.len.ok_or(TypeCodecError::MissingContext("len"))?;
        if self.0.len() != len {
            return Err(TypeCodecError::ContextMismatch("array length does not match ctx.len"));
        }

        for value in &self.0 {
            value.encode(buf)?;
        }
        Ok(())
    }

    fn decode_with_ctx(buf: &mut &[u8], ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let len = ctx.len.ok_or(TypeCodecError::MissingContext("len"))?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(T::decode(buf)?);
        }
        Ok(Self(values))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixedArray<T>(pub Vec<T>);

impl<T> PrefixedArray<T> {
    pub const fn new(values: Vec<T>) -> Self {
        Self(values)
    }
}
impl<T: Codec> Codec for PrefixedArray<T> {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.encode_with_ctx(buf, &Ctx::none())
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
        Self::decode_with_ctx(buf, &Ctx::none())
    }
}
impl<T> ContextualCodec<Ctx> for PrefixedArray<T>
where
    T: Codec,
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, _ctx: &Ctx) -> Result<(), TypeCodecError> {
        let len = i32::try_from(self.0.len()).map_err(|_| TypeCodecError::InvalidArrayLength(i32::MAX))?;
        buf.write_varint(len)?;
        for value in &self.0 {
            value.encode(buf)?;
        }
        Ok(())
    }

    fn decode_with_ctx(buf: &mut &[u8], _ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let len = buf.read_varint()?;
        if len < 0 {
            return Err(TypeCodecError::InvalidArrayLength(len));
        }
        let len = len as usize;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(T::decode(buf)?);
        }
        Ok(Self(values))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteArray(pub Vec<u8>);

impl ByteArray {
    pub const fn new(values: Vec<u8>) -> Self {
        Self(values)
    }
}

impl ContextualCodec<Ctx> for ByteArray {
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, ctx: &Ctx) -> Result<(), TypeCodecError> {
        let len = ctx.len.ok_or(TypeCodecError::MissingContext("len"))?;
        if self.0.len() != len {
            return Err(TypeCodecError::ContextMismatch("byte array length does not match ctx.len"));
        }
        buf.extend_from_slice(&self.0);
        Ok(())
    }

    fn decode_with_ctx(buf: &mut &[u8], ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let len = ctx.len.ok_or(TypeCodecError::MissingContext("len"))?;
        let bytes = buf
            .get(..len)
            .ok_or(TypeCodecError::EndOfBuffer(buf.len(), len))?
            .to_vec();
        *buf = &buf[len..];
        Ok(Self(bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdOr<T> {
    Inline(T),
    RegistryId(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdSet {
    Tag(Identifier),
    InlineIds(Vec<i32>),
}

impl ContextualCodec<Ctx> for IdSet {
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, _ctx: &Ctx) -> Result<(), TypeCodecError> {
        match self {
            Self::Tag(tag_name) => {
                buf.write_varint(0)?;
                tag_name.encode(buf)
            }
            Self::InlineIds(ids) => {
                let len = i32::try_from(ids.len()).map_err(|_| TypeCodecError::InvalidArrayLength(i32::MAX))?;
                let ty = len
                    .checked_add(1)
                    .ok_or(TypeCodecError::InvalidArrayLength(len))?;
                buf.write_varint(ty)?;
                for id in ids {
                    buf.write_varint(*id)?;
                }
                Ok(())
            }
        }
    }

    fn decode_with_ctx(buf: &mut &[u8], _ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let ty = buf.read_varint()?;
        if ty < 0 {
            return Err(TypeCodecError::InvalidArrayLength(ty));
        }
        if ty == 0 {
            Ok(Self::Tag(Identifier::decode(buf)?))
        } else {
            let len = usize::try_from(ty - 1).map_err(|_| TypeCodecError::InvalidArrayLength(ty))?;
            let mut ids = Vec::with_capacity(len);
            for _ in 0..len {
                ids.push(buf.read_varint()?);
            }
            Ok(Self::InlineIds(ids))
        }
    }
}

impl<T> ContextualCodec<Ctx> for IdOr<T>
where
    T: Codec,
{
    fn encode_with_ctx(&self, buf: &mut Vec<u8>, _ctx: &Ctx) -> Result<(), TypeCodecError> {
        match self {
            Self::Inline(value) => {
                buf.write_varint(0)?;
                value.encode(buf)
            }
            Self::RegistryId(id) => {
                if *id < 0 {
                    return Err(TypeCodecError::InvalidIdOrValue(*id));
                }
                buf.write_varint(*id + 1)?;
                Ok(())
            }
        }
    }

    fn decode_with_ctx(buf: &mut &[u8], _ctx: &Ctx) -> Result<Self, TypeCodecError> {
        let raw = buf.read_varint()?;
        if raw < 0 {
            return Err(TypeCodecError::InvalidIdOrValue(raw));
        }
        if raw == 0 {
            Ok(Self::Inline(T::decode(buf)?))
        } else {
            Ok(Self::RegistryId(raw - 1)) // 游标
        }
    }
}


// 对于X Enum / EnumSet (N) : 遇到一个写一个，用泛型史到飞起来

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEvent {
    pub sound_name: Identifier,
    pub fixed_range: Option<Float>,
}

impl SoundEvent {
    pub const fn new(sound_name: Identifier, fixed_range: Option<Float>) -> Self {
        Self {
            sound_name,
            fixed_range,
        }
    }
    pub fn has_fixed_range(&self) -> bool {
        self.fixed_range.is_some()
    }
}

impl Codec for SoundEvent {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
        self.sound_name.encode(buf)?;
        if let Some(fixed_range) = &self.fixed_range {
            true.encode(buf)?;
            fixed_range.encode(buf)
        } else {
            false.encode(buf)
        }
    }

    fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError>
    where
        Self: Sized
    {
        let sound_name = Identifier::decode(buf)?;
        let has_fixed_range = bool::decode(buf)?;

        if has_fixed_range {
            Ok(SoundEvent {
                sound_name,
                fixed_range: Some(Float::decode(buf)?)
            })
        } else {
            Ok(SoundEvent {
                sound_name,
                fixed_range: None
            })
        }
    }
}
