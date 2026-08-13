//! Context and optional protocol values whose wire representation depends on
//! their enclosing packet or data structure.

use crate::{ContextualCodec, TypeCodec};
use mcproto_codec::error::{CodecError, CodecKind, CodecOperation, InvalidEncodingReason};

/// External information required to encode or decode a contextual value.
///
/// The initial context records whether the current field is present. This is
/// intended for protocol fields described as `Optional X`, where no presence
/// marker is encoded and the field's presence must be known from its enclosing
/// packet or data structure.
///
/// A `Context` does not consume or produce any bytes. The enclosing codec must
/// derive it from already-known protocol state and pass it to
/// [`ContextualCodec`](crate::ContextualCodec).
///
/// # Examples
///
/// ```
/// use mcproto_types::contextual::Context;
///
/// let has_signature = true; // Derived from an earlier packet field.
/// let context = Context::new(has_signature);
/// assert!(context.is_present());
/// assert!(!Context::absent().is_present());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Context {
    present: bool,
}

impl Context {
    /// Context for a field that is present on the wire.
    pub const PRESENT: Self = Self::new(true);

    /// Context for a field that occupies zero bytes on the wire.
    pub const ABSENT: Self = Self::new(false);

    /// Creates a context from a presence condition determined by the enclosing
    /// protocol structure.
    #[must_use]
    pub const fn new(present: bool) -> Self {
        Self { present }
    }

    /// Creates context for a field that is present on the wire.
    #[must_use]
    pub const fn present() -> Self {
        Self::PRESENT
    }

    /// Creates context for a field that occupies zero bytes on the wire.
    #[must_use]
    pub const fn absent() -> Self {
        Self::ABSENT
    }

    /// Returns whether the contextual field is present on the wire.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        self.present
    }
}

/// A context-controlled optional value of protocol type `T`.
///
/// `Optional<T>` stores an [`Option<T>`], but it does not encode a presence
/// marker. When the supplied [`Context`] is present, the inner `T` is encoded
/// exactly as its [`TypeCodec`] implementation specifies. When the context is
/// absent, the value occupies zero bytes.
///
/// The caller must derive the context from the enclosing packet or data
/// structure. This type therefore implements [`ContextualCodec`], not
/// [`TypeCodec`]. A value/context mismatch is reported as an encoding error;
/// it is never silently discarded.
///
/// # Examples
///
/// ```
/// use mcproto_types::{ContextualCodec, TypeCodec, basic::UnsignedByte};
/// use mcproto_types::contextual::{Context, Optional};
///
/// let value = Optional::some(UnsignedByte(0xab));
/// let mut encoded = Vec::new();
/// value.encode_with_context(&mut encoded, &Context::present())?;
/// assert_eq!(encoded, [0xab]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(
///     Optional::<UnsignedByte>::decode_with_context(&mut input, &Context::present())?,
///     value,
/// );
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Optional<T>(
    /// The optional value held in memory.
    pub Option<T>,
);

impl<T> Optional<T> {
    /// Creates an optional value that is present.
    #[must_use]
    pub const fn some(value: T) -> Self {
        Self(Some(value))
    }

    /// Creates an optional value that is absent.
    #[must_use]
    pub const fn none() -> Self {
        Self(None)
    }

    /// Returns whether this wrapper contains a value.
    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.0.is_some()
    }

    /// Returns whether this wrapper contains no value.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the contained value by reference, if present.
    #[must_use]
    pub const fn as_ref(&self) -> Optional<&T> {
        Optional(self.0.as_ref())
    }

    /// Extracts the wrapped [`Option<T>`].
    #[must_use]
    pub fn into_option(self) -> Option<T> {
        self.0
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(value: Optional<T>) -> Self {
        value.0
    }
}

impl<T> ContextualCodec for Optional<T>
where
    T: TypeCodec,
{
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &Context,
    ) -> Result<(), CodecError> {
        match (context.is_present(), self.0.as_ref()) {
            (true, Some(value)) => value
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::Optional)),
            (false, None) => Ok(()),
            (context_present, value) => Err(CodecError::invalid_encoding_for_operation(
                CodecKind::Optional,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::OptionalValueMismatch {
                    context_present,
                    value_present: value.is_some(),
                },
            )),
        }
    }

    fn decode_with_context(
        reader: &mut impl std::io::Read,
        context: &Context,
    ) -> Result<Self, CodecError> {
        if context.is_present() {
            T::decode(reader)
                .map(|value| Self::some(value))
                .map_err(|error| error.with_context(CodecKind::Optional))
        } else {
            Ok(Self::none())
        }
    }
}
