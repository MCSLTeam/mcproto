use crate::{Codec, ContextualCodec, Ctx, TypeCodecError};
use mcproto_codec::{VarIntRead, VarIntWrite};

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


// 对于X Enum / EnumSet (N) : 遇到一个写一个，用泛型史到飞起来

