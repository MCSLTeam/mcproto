use crate::{Codec, ContextualCodec, Ctx, TypeCodecError};

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
