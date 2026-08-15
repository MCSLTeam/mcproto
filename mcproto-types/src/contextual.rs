//! Context and protocol values whose wire representation depends on their
//! enclosing packet or data structure.

use crate::{
    ContextualCodec, TypeCodec,
    basic::{Boolean, Identifier},
};
use mcproto_codec::error::{
    CodecError, CodecKind, CodecOperation, ContextRequirement, InvalidEncodingReason,
};
use mcproto_codec::io::{read_exact_counted, write_all_counted};
use mcproto_codec::varint::{VarIntRead, VarIntWrite};

/// External information required to encode or decode a contextual value.
///
/// The context can record whether the current field is present, the length of
/// an array field, and child contexts for array elements. This supports
/// protocol fields described as `Optional X` or `Array of X`, where the
/// required information is known from the enclosing packet or data structure.
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    presence: Option<bool>,
    array_length: Option<usize>,
    element_contexts: Option<Box<[Context]>>,
}

impl Context {
    /// Context for a field that is present on the wire.
    pub const PRESENT: Self = Self {
        presence: Some(true),
        array_length: None,
        element_contexts: None,
    };

    /// Context for a field that occupies zero bytes on the wire.
    pub const ABSENT: Self = Self {
        presence: Some(false),
        array_length: None,
        element_contexts: None,
    };

    /// Creates a context from a presence condition determined by the enclosing
    /// protocol structure.
    #[must_use]
    pub const fn new(present: bool) -> Self {
        Self {
            presence: Some(present),
            array_length: None,
            element_contexts: None,
        }
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

    /// Creates context containing the number of elements in an array field.
    #[must_use]
    pub const fn for_array_length(length: usize) -> Self {
        Self {
            presence: None,
            array_length: Some(length),
            element_contexts: None,
        }
    }

    /// Adds an array length to this context, preserving any presence state.
    #[must_use]
    pub fn with_array_length(self, length: usize) -> Self {
        Self {
            presence: self.presence,
            array_length: Some(length),
            element_contexts: self.element_contexts,
        }
    }

    /// Adds a child context for each array element.
    ///
    /// When child contexts are not supplied, an array passes its own context
    /// to every element. Supplying child contexts is required when different
    /// elements have different contextual metadata or when arrays are nested.
    #[must_use]
    pub fn with_element_contexts(self, contexts: impl IntoIterator<Item = Context>) -> Self {
        Self {
            presence: self.presence,
            array_length: self.array_length,
            element_contexts: Some(contexts.into_iter().collect()),
        }
    }

    /// Returns the explicitly supplied presence state, if one exists.
    #[must_use]
    pub const fn presence(&self) -> Option<bool> {
        self.presence
    }

    /// Returns the contextual array length, if one exists.
    #[must_use]
    pub const fn array_length(&self) -> Option<usize> {
        self.array_length
    }

    fn element_context(
        &self,
        index: usize,
        operation: CodecOperation,
    ) -> Result<&Context, CodecError> {
        match &self.element_contexts {
            Some(contexts) => contexts.get(index).ok_or_else(|| {
                missing_context(
                    CodecKind::Array,
                    operation,
                    ContextRequirement::ElementContext,
                )
            }),
            None => Ok(self),
        }
    }

    /// Returns whether the contextual field is present on the wire.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self.presence, Some(true))
    }
}

fn missing_context(
    codec: CodecKind,
    operation: CodecOperation,
    required: ContextRequirement,
) -> CodecError {
    CodecError::invalid_encoding_for_operation(
        codec,
        operation,
        0,
        InvalidEncodingReason::MissingContext { required },
    )
}

/// A sequence of protocol values whose element count is supplied by context.
///
/// `Array<T>` has no wire length prefix. The enclosing packet must supply the
/// number of elements through [`Context::for_array_length`] or
/// [`Context::with_array_length`]. Exactly that many values are encoded or
/// decoded. A zero length therefore produces and consumes zero bytes.
///
/// The total byte size is not necessarily `length * fixed_size`: if `T` has a
/// variable-size encoding, each element may occupy a different number of
/// bytes.
///
/// # Examples
///
/// ```
/// use mcproto_types::{ContextualCodec, TypeCodec, basic::UnsignedByte};
/// use mcproto_types::contextual::{Array, Context};
///
/// let values = Array(vec![UnsignedByte(1), UnsignedByte(2)]);
/// let mut encoded = Vec::new();
/// values.encode_with_context(&mut encoded, &Context::for_array_length(2))?;
/// assert_eq!(encoded, [1, 2]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(
///     Array::<UnsignedByte>::decode_with_context(
///         &mut input,
///         &Context::for_array_length(2),
///     )?,
///     values,
/// );
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Array<T>(
    /// The array elements.
    pub Vec<T>,
);

impl<T> Array<T> {
    /// Creates an array from its elements.
    #[must_use]
    pub const fn new(values: Vec<T>) -> Self {
        Self(values)
    }

    /// Returns the number of elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the array contains no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the elements as a slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Extracts the underlying vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for Array<T> {
    fn from(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> From<Array<T>> for Vec<T> {
    fn from(values: Array<T>) -> Self {
        values.0
    }
}

impl<T> ContextualCodec for Array<T>
where
    T: ContextualCodec,
{
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &Context,
    ) -> Result<(), CodecError> {
        let expected = context.array_length().ok_or_else(|| {
            missing_context(
                CodecKind::Array,
                CodecOperation::Write,
                ContextRequirement::Length,
            )
        })?;
        if self.len() != expected {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::Array,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::ArrayLengthMismatch {
                    expected,
                    actual: self.len(),
                },
            ));
        }

        for (index, value) in self.0.iter().enumerate() {
            let element_context = context.element_context(index, CodecOperation::Write)?;
            value
                .encode_with_context(writer, element_context)
                .map_err(|error| error.with_context(CodecKind::Array))?;
        }
        Ok(())
    }

    fn decode_with_context(
        reader: &mut impl std::io::Read,
        context: &Context,
    ) -> Result<Self, CodecError> {
        let length = context.array_length().ok_or_else(|| {
            missing_context(
                CodecKind::Array,
                CodecOperation::Read,
                ContextRequirement::Length,
            )
        })?;
        let mut values = Vec::with_capacity(length);
        for index in 0..length {
            let element_context = context.element_context(index, CodecOperation::Read)?;
            values.push(
                T::decode_with_context(reader, element_context)
                    .map_err(|error| error.with_context(CodecKind::Array))?,
            );
        }
        Ok(Self(values))
    }
}

/// A raw sequence of bytes whose length is supplied by context.
///
/// A `ByteArray` has no wire length prefix. Its meaning and number of bytes
/// are determined by the enclosing packet or data structure, which supplies
/// the length through [`Context::for_array_length`] or
/// [`Context::with_array_length`]. It is encoded as exactly that many bytes:
///
/// ```text
/// byte[0] + byte[1] + ... + byte[length - 1]
/// ```
///
/// This differs from [`PrefixedArray`], which stores its own VarInt length,
/// and from [`Array`], which supports arbitrary contextual element codecs.
/// `ByteArray` writes and reads its byte buffer in one operation and does not
/// use array element contexts.
///
/// [Minecraft protocol Byte Array]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Byte_Array
///
/// # Examples
///
/// ```
/// use mcproto_types::ContextualCodec;
/// use mcproto_types::contextual::{ByteArray, Context};
///
/// let value = ByteArray(vec![0xde, 0xad, 0xbe, 0xef]);
/// let context = Context::for_array_length(4);
/// let mut encoded = Vec::new();
/// value.encode_with_context(&mut encoded, &context)?;
/// assert_eq!(encoded, [0xde, 0xad, 0xbe, 0xef]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(ByteArray::decode_with_context(&mut input, &context)?, value);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ByteArray(
    /// The raw bytes.
    pub Vec<u8>,
);

impl ByteArray {
    /// Creates a byte array from raw bytes.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the number of bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether this byte array is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the bytes as a slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Extracts the underlying byte vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for ByteArray {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<ByteArray> for Vec<u8> {
    fn from(bytes: ByteArray) -> Self {
        bytes.0
    }
}

impl ContextualCodec for ByteArray {
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &Context,
    ) -> Result<(), CodecError> {
        let expected = context.array_length().ok_or_else(|| {
            missing_context(
                CodecKind::ByteArray,
                CodecOperation::Write,
                ContextRequirement::Length,
            )
        })?;
        if self.len() != expected {
            return Err(CodecError::invalid_encoding_for_operation(
                CodecKind::ByteArray,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::ArrayLengthMismatch {
                    expected,
                    actual: self.len(),
                },
            ));
        }

        write_all_counted(writer, &self.0, CodecKind::ByteArray, 0)
    }

    fn decode_with_context(
        reader: &mut impl std::io::Read,
        context: &Context,
    ) -> Result<Self, CodecError> {
        let length = context.array_length().ok_or_else(|| {
            missing_context(
                CodecKind::ByteArray,
                CodecOperation::Read,
                ContextRequirement::Length,
            )
        })?;
        let mut bytes = vec![0; length];
        read_exact_counted(reader, &mut bytes, CodecKind::ByteArray, 0)?;
        Ok(Self(bytes))
    }
}

/// A sequence prefixed by its element count as a VarInt.
///
/// The [Minecraft protocol Prefixed Array] wire representation is a
/// non-negative [`VarInt`] length followed by exactly that many `T` values:
///
/// ```text
/// VarInt(length) + T[0] + T[1] + ... + T[length - 1]
/// ```
///
/// A zero length is encoded as `0x00` and has no element payload. Since the
/// prefix is a signed 32-bit VarInt, arrays cannot contain more than
/// 2,147,483,647 elements. This type supports all context-independent
/// protocol values through `T: TypeCodec`.
///
/// [Minecraft protocol Prefixed Array]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Prefixed_Array
/// [`VarInt`]: crate::basic::VarInt
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::UnsignedByte};
/// use mcproto_types::contextual::PrefixedArray;
///
/// let values = PrefixedArray(vec![UnsignedByte(1), UnsignedByte(2)]);
/// let mut encoded = Vec::new();
/// values.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x02, 0x01, 0x02]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(PrefixedArray::<UnsignedByte>::decode(&mut input)?, values);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrefixedArray<T>(
    /// The array elements.
    pub Vec<T>,
);

impl<T> PrefixedArray<T> {
    /// Creates a length-prefixed array from its elements.
    #[must_use]
    pub const fn new(values: Vec<T>) -> Self {
        Self(values)
    }

    /// Returns the number of elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the array contains no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the elements as a slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Extracts the underlying vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for PrefixedArray<T> {
    fn from(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> From<PrefixedArray<T>> for Vec<T> {
    fn from(values: PrefixedArray<T>) -> Self {
        values.0
    }
}

impl<T> TypeCodec for PrefixedArray<T>
where
    T: TypeCodec,
{
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        let length = i32::try_from(self.len()).map_err(|_| {
            CodecError::invalid_encoding_for_operation(
                CodecKind::PrefixedArray,
                CodecOperation::Write,
                0,
                InvalidEncodingReason::LengthOutOfRange {
                    max: i32::MAX as usize,
                    actual: self.len(),
                },
            )
        })?;

        writer
            .write_varint(length)
            .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        for value in &self.0 {
            value
                .encode(writer)
                .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let (length, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::PrefixedArray))?;
        if length < 0 {
            return Err(CodecError::invalid_encoding(
                CodecKind::PrefixedArray,
                prefix_size,
                InvalidEncodingReason::NegativeLength { value: length },
            ));
        }

        let mut values = Vec::new();
        for _ in 0..length as usize {
            values.push(
                T::decode(reader).map_err(|error| error.with_context(CodecKind::PrefixedArray))?,
            );
        }
        Ok(Self(values))
    }
}

/// A context-controlled optional value of protocol type `T`.
///
/// `Optional<T>` stores an [`Option<T>`], but it does not encode a presence
/// marker. When the supplied [`Context`] is present, the inner `T` is encoded
/// using its [`ContextualCodec`] implementation. When the context is absent,
/// the value occupies zero bytes.
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
    T: ContextualCodec,
{
    fn encode_with_context(
        &self,
        writer: &mut impl std::io::Write,
        context: &Context,
    ) -> Result<(), CodecError> {
        let context_present = context.presence().ok_or_else(|| {
            missing_context(
                CodecKind::Optional,
                CodecOperation::Write,
                ContextRequirement::Presence,
            )
        })?;
        match (context_present, self.0.as_ref()) {
            (true, Some(value)) => value
                .encode_with_context(writer, context)
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
        match context.presence().ok_or_else(|| {
            missing_context(
                CodecKind::Optional,
                CodecOperation::Read,
                ContextRequirement::Presence,
            )
        })? {
            true => T::decode_with_context(reader, context)
                .map(Self::some)
                .map_err(|error| error.with_context(CodecKind::Optional)),
            false => Ok(Self::none()),
        }
    }
}

/// An optional value prefixed by a boolean presence marker.
///
/// The wire format is a [`Boolean`] followed by `T` when the boolean is true:
///
/// ```text
/// Boolean(is present) + (is present ? T : nothing)
/// ```
///
/// Unlike [`Optional<T>`], this type implements [`TypeCodec`] because its wire
/// representation contains its own presence marker. The marker is `0x01`
/// when the wrapped value is [`Some`](Option::Some), and `0x00` when it is
/// [`None`](Option::None).
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::UnsignedByte};
/// use mcproto_types::contextual::PrefixedOptional;
///
/// let value = PrefixedOptional::some(UnsignedByte(0xab));
/// let mut encoded = Vec::new();
/// value.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x01, 0xab]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(PrefixedOptional::<UnsignedByte>::decode(&mut input)?, value);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrefixedOptional<T>(
    /// The optional value and its context-controlled encoding behavior.
    pub Optional<T>,
);

impl<T> PrefixedOptional<T> {
    /// Creates a prefixed optional containing `value`.
    #[must_use]
    pub const fn some(value: T) -> Self {
        Self(Optional::some(value))
    }

    /// Creates a prefixed optional with no value.
    #[must_use]
    pub const fn none() -> Self {
        Self(Optional::none())
    }

    /// Returns whether the prefixed optional contains a value.
    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.0.is_some()
    }

    /// Returns whether the prefixed optional contains no value.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the contained value by reference, if present.
    #[must_use]
    pub const fn as_ref(&self) -> PrefixedOptional<&T> {
        PrefixedOptional(self.0.as_ref())
    }

    /// Extracts the wrapped [`Option<T>`].
    #[must_use]
    pub fn into_option(self) -> Option<T> {
        self.0.into_option()
    }
}

impl<T> From<Option<T>> for PrefixedOptional<T> {
    fn from(value: Option<T>) -> Self {
        Self(value.into())
    }
}

impl<T> From<Optional<T>> for PrefixedOptional<T> {
    fn from(value: Optional<T>) -> Self {
        Self(value)
    }
}

impl<T> From<PrefixedOptional<T>> for Option<T> {
    fn from(value: PrefixedOptional<T>) -> Self {
        value.into_option()
    }
}

impl<T> TypeCodec for PrefixedOptional<T>
where
    T: TypeCodec,
{
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        let context = Context::new(self.is_some());
        Boolean(self.is_some())
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::PrefixedOptional))?;
        self.0
            .encode_with_context(writer, &context)
            .map_err(|error| error.with_context(CodecKind::PrefixedOptional))
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let present = Boolean::decode(reader)
            .map_err(|error| error.with_context(CodecKind::PrefixedOptional))?;
        Optional::decode_with_context(reader, &Context::new(present.0))
            .map(Self)
            .map_err(|error| error.with_context(CodecKind::PrefixedOptional))
    }
}

/// A protocol value represented either by registry ID or by an inline `T`.
///
/// The [Minecraft protocol ID or X] wire representation begins with a
/// [`VarInt`] selector:
///
/// - `0` means that a value of type `T` follows inline;
/// - a positive value `n` refers to registry ID `n - 1` and has no inline
///   payload.
///
/// ```text
/// VarInt(0) + T              // Inline value
/// VarInt(registry_id + 1)    // Registry reference
/// ```
///
/// Registry IDs held by this type are the actual zero-based IDs, not their
/// incremented wire selectors. Valid IDs range from `0` through
/// `i32::MAX - 1`. The registry itself is implied by the enclosing packet or
/// field definition; it does not need to be stored in [`Context`] because it
/// does not affect this value's byte layout.
///
/// [Minecraft protocol ID or X]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#ID_or_X
/// [`VarInt`]: crate::basic::VarInt
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::UnsignedByte};
/// use mcproto_types::contextual::IdOr;
///
/// let inline = IdOr::inline(UnsignedByte(0xab));
/// let mut encoded = Vec::new();
/// inline.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x00, 0xab]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(IdOr::<UnsignedByte>::decode(&mut input)?, inline);
///
/// let reference = IdOr::<UnsignedByte>::id(4);
/// let mut encoded = Vec::new();
/// reference.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x05]);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdOr<T> {
    /// A zero-based ID in the registry implied by the enclosing field.
    Id(i32),
    /// A complete value encoded inline after a zero selector.
    Inline(T),
}

impl<T> IdOr<T> {
    /// Creates a registry reference from its actual zero-based ID.
    #[must_use]
    pub const fn id(id: i32) -> Self {
        Self::Id(id)
    }

    /// Creates an inline value.
    #[must_use]
    pub const fn inline(value: T) -> Self {
        Self::Inline(value)
    }

    /// Returns whether this value is a registry reference.
    #[must_use]
    pub const fn is_id(&self) -> bool {
        matches!(self, Self::Id(_))
    }

    /// Returns whether this value is defined inline.
    #[must_use]
    pub const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    /// Returns the zero-based registry ID, if this is a reference.
    #[must_use]
    pub const fn registry_id(&self) -> Option<i32> {
        match self {
            Self::Id(id) => Some(*id),
            Self::Inline(_) => None,
        }
    }

    /// Returns the inline value by reference, if present.
    #[must_use]
    pub const fn inline_value(&self) -> Option<&T> {
        match self {
            Self::Id(_) => None,
            Self::Inline(value) => Some(value),
        }
    }

    /// Borrows the inline value while preserving registry references.
    #[must_use]
    pub const fn as_ref(&self) -> IdOr<&T> {
        match self {
            Self::Id(id) => IdOr::Id(*id),
            Self::Inline(value) => IdOr::Inline(value),
        }
    }

    /// Extracts the inline value, if present.
    #[must_use]
    pub fn into_inline(self) -> Option<T> {
        match self {
            Self::Id(_) => None,
            Self::Inline(value) => Some(value),
        }
    }
}

impl<T> From<T> for IdOr<T> {
    fn from(value: T) -> Self {
        Self::Inline(value)
    }
}

impl<T> TypeCodec for IdOr<T>
where
    T: TypeCodec,
{
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        match self {
            Self::Id(id) => {
                let selector = id.checked_add(1).filter(|_| *id >= 0).ok_or_else(|| {
                    CodecError::invalid_encoding_for_operation(
                        CodecKind::IdOr,
                        CodecOperation::Write,
                        0,
                        InvalidEncodingReason::InvalidRegistryId {
                            value: *id,
                            max: i32::MAX - 1,
                        },
                    )
                })?;
                writer
                    .write_varint(selector)
                    .map_err(|error| error.with_context(CodecKind::IdOr))
            }
            Self::Inline(value) => {
                writer
                    .write_varint(0)
                    .map_err(|error| error.with_context(CodecKind::IdOr))?;
                value
                    .encode(writer)
                    .map_err(|error| error.with_context(CodecKind::IdOr))
            }
        }
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let (selector, prefix_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::IdOr))?;
        match selector {
            0 => T::decode(reader)
                .map(Self::Inline)
                .map_err(|error| error.with_context(CodecKind::IdOr)),
            1.. => Ok(Self::Id(selector - 1)),
            _ => Err(CodecError::invalid_encoding(
                CodecKind::IdOr,
                prefix_size,
                InvalidEncodingReason::InvalidIdOrSelector { value: selector },
            )),
        }
    }
}

/// A set of registry IDs represented inline or by reference to a tag.
///
/// The registry itself is implied by the enclosing packet or field. The
/// [Minecraft protocol ID Set] wire representation starts with a [`VarInt`]
/// type value:
///
/// - `0` is followed by an [`Identifier`] naming a registry tag;
/// - a positive value `n` is followed by `n - 1` registry IDs encoded as
///   VarInts.
///
/// ```text
/// VarInt(0) + Identifier(tag_name)
/// VarInt(ids.len() + 1) + VarInt(ids[0]) + ... + VarInt(ids[len - 1])
/// ```
///
/// An empty inline set therefore uses type value `1`. Registry IDs must be
/// non-negative, and an inline set may contain at most `i32::MAX - 1` IDs.
///
/// [Minecraft protocol ID Set]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#ID_Set
/// [`VarInt`]: crate::basic::VarInt
///
/// # Examples
///
/// ```
/// use mcproto_types::{TypeCodec, basic::Identifier};
/// use mcproto_types::contextual::IdSet;
///
/// let inline = IdSet::inline(vec![3, 7]);
/// let mut encoded = Vec::new();
/// inline.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x03, 0x03, 0x07]);
///
/// let mut input = encoded.as_slice();
/// assert_eq!(IdSet::decode(&mut input)?, inline);
///
/// let tagged = IdSet::tag(Identifier::new("minecraft:logs")?);
/// assert!(tagged.is_tag());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdSet {
    /// A named set of IDs defined by a registry tag.
    Tag(Identifier),
    /// An ad-hoc set of zero-based registry IDs enumerated inline.
    Inline(Vec<i32>),
}

impl IdSet {
    /// Creates an ID set that refers to a registry tag.
    #[must_use]
    pub const fn tag(tag_name: Identifier) -> Self {
        Self::Tag(tag_name)
    }

    /// Creates an ID set containing inline registry IDs.
    #[must_use]
    pub const fn inline(ids: Vec<i32>) -> Self {
        Self::Inline(ids)
    }

    /// Returns whether this set refers to a registry tag.
    #[must_use]
    pub const fn is_tag(&self) -> bool {
        matches!(self, Self::Tag(_))
    }

    /// Returns whether this set enumerates registry IDs inline.
    #[must_use]
    pub const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    /// Returns the registry tag name, if this is a tag reference.
    #[must_use]
    pub const fn tag_name(&self) -> Option<&Identifier> {
        match self {
            Self::Tag(tag_name) => Some(tag_name),
            Self::Inline(_) => None,
        }
    }

    /// Returns the inline registry IDs, if present.
    #[must_use]
    pub const fn ids(&self) -> Option<&[i32]> {
        match self {
            Self::Tag(_) => None,
            Self::Inline(ids) => Some(ids.as_slice()),
        }
    }

    /// Extracts the registry tag name, if this is a tag reference.
    #[must_use]
    pub fn into_tag(self) -> Option<Identifier> {
        match self {
            Self::Tag(tag_name) => Some(tag_name),
            Self::Inline(_) => None,
        }
    }

    /// Extracts the inline registry IDs, if present.
    #[must_use]
    pub fn into_ids(self) -> Option<Vec<i32>> {
        match self {
            Self::Tag(_) => None,
            Self::Inline(ids) => Some(ids),
        }
    }
}

impl From<Identifier> for IdSet {
    fn from(tag_name: Identifier) -> Self {
        Self::Tag(tag_name)
    }
}

impl From<Vec<i32>> for IdSet {
    fn from(ids: Vec<i32>) -> Self {
        Self::Inline(ids)
    }
}

impl TypeCodec for IdSet {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        match self {
            Self::Tag(tag_name) => {
                writer
                    .write_varint(0)
                    .map_err(|error| error.with_context(CodecKind::IdSet))?;
                tag_name
                    .encode(writer)
                    .map_err(|error| error.with_context(CodecKind::IdSet))
            }
            Self::Inline(ids) => {
                if let Some(id) = ids.iter().copied().find(|id| *id < 0) {
                    return Err(CodecError::invalid_encoding_for_operation(
                        CodecKind::IdSet,
                        CodecOperation::Write,
                        0,
                        InvalidEncodingReason::InvalidRegistryId {
                            value: id,
                            max: i32::MAX,
                        },
                    ));
                }
                let type_value = i32::try_from(ids.len())
                    .ok()
                    .and_then(|length| length.checked_add(1))
                    .ok_or_else(|| {
                        CodecError::invalid_encoding_for_operation(
                            CodecKind::IdSet,
                            CodecOperation::Write,
                            0,
                            InvalidEncodingReason::LengthOutOfRange {
                                max: (i32::MAX - 1) as usize,
                                actual: ids.len(),
                            },
                        )
                    })?;

                writer
                    .write_varint(type_value)
                    .map_err(|error| error.with_context(CodecKind::IdSet))?;
                for id in ids {
                    writer
                        .write_varint(*id)
                        .map_err(|error| error.with_context(CodecKind::IdSet))?;
                }
                Ok(())
            }
        }
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let (type_value, type_size) = reader
            .read_varint_with_size()
            .map_err(|error| error.with_context(CodecKind::IdSet))?;
        match type_value {
            0 => Identifier::decode(reader)
                .map(Self::Tag)
                .map_err(|error| error.with_context(CodecKind::IdSet)),
            1.. => {
                let length = (type_value - 1) as usize;
                let mut bytes_processed = type_size;
                let mut ids = Vec::new();
                for _ in 0..length {
                    let (id, id_size) = reader
                        .read_varint_with_size()
                        .map_err(|error| error.with_context(CodecKind::IdSet))?;
                    bytes_processed += id_size;
                    if id < 0 {
                        return Err(CodecError::invalid_encoding(
                            CodecKind::IdSet,
                            bytes_processed,
                            InvalidEncodingReason::InvalidRegistryId {
                                value: id,
                                max: i32::MAX,
                            },
                        ));
                    }
                    ids.push(id);
                }
                Ok(Self::Inline(ids))
            }
            _ => Err(CodecError::invalid_encoding(
                CodecKind::IdSet,
                type_size,
                InvalidEncodingReason::InvalidIdSetType { value: type_value },
            )),
        }
    }
}
