//! Minecraft protocol direct chat types.

use mcproto_codec::error::{CodecError, CodecKind};

use crate::{
    ProtocolEnum, TypeCodec,
    basic::{PrefixedString, VarInt},
    contextual::PrefixedArray,
    nbt::Nbt,
};

/// A value inserted into a chat decoration's translated message.
///
/// Each parameter is encoded as a [`VarInt`]. Its position in the parameter
/// array determines the corresponding argument passed to the translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ProtocolEnum)]
#[protocol_enum(repr = VarInt)]
pub enum ChatTypeParameter {
    /// The component representing the message sender.
    Sender = 0,
    /// The component representing the message target.
    Target = 1,
    /// The component containing the message content.
    Content = 2,
}

/// Controls how one form of a direct chat message is decorated.
///
/// A decoration is encoded as its translation key, a length-prefixed array of
/// [`ChatTypeParameter`] values, and an [`Nbt`] style. The style is always
/// present on the wire; an unstyled decoration uses an empty NBT compound.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatDecoration {
    /// Translation key used to format the message.
    pub translation_key: PrefixedString,
    /// Ordered arguments supplied to the translated message.
    pub parameters: PrefixedArray<ChatTypeParameter>,
    /// Text style encoded as network NBT.
    pub style: Nbt,
}

impl ChatDecoration {
    /// Creates a chat decoration from its translation key, parameters, and style.
    #[must_use]
    pub const fn new(
        translation_key: PrefixedString,
        parameters: PrefixedArray<ChatTypeParameter>,
        style: Nbt,
    ) -> Self {
        Self {
            translation_key,
            parameters,
            style,
        }
    }
}

impl TypeCodec for ChatDecoration {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        self.translation_key
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ChatDecoration))?;
        self.parameters
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ChatDecoration))?;
        self.style
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ChatDecoration))
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let translation_key = PrefixedString::decode(reader)
            .map_err(|error| error.with_context(CodecKind::ChatDecoration))?;
        let parameters = PrefixedArray::<ChatTypeParameter>::decode(reader)
            .map_err(|error| error.with_context(CodecKind::ChatDecoration))?;
        let style =
            Nbt::decode(reader).map_err(|error| error.with_context(CodecKind::ChatDecoration))?;
        Ok(Self::new(translation_key, parameters, style))
    }
}

/// Describes a direct chat type that a message can be sent with.
///
/// The chat decoration is encoded first and the narration decoration second:
///
/// ```text
/// ChatDecoration(chat) + ChatDecoration(narration)
/// ```
///
/// Both fields use the same decoration structure, but may use different
/// translation keys, parameter orders, and styles.
///
/// # Examples
///
/// ```
/// use fastnbt::nbt;
/// use mcproto_types::{
///     ChatDecoration, ChatType, ChatTypeParameter, Nbt, PrefixedArray,
///     PrefixedString, TypeCodec,
/// };
///
/// let value = ChatType::new(
///     ChatDecoration::new(
///         PrefixedString("chat.type.text".into()),
///         PrefixedArray(vec![ChatTypeParameter::Sender, ChatTypeParameter::Content]),
///         Nbt(nbt!({})),
///     ),
///     ChatDecoration::new(
///         PrefixedString("chat.type.text.narrate".into()),
///         PrefixedArray(vec![ChatTypeParameter::Sender, ChatTypeParameter::Content]),
///         Nbt(nbt!({})),
///     ),
/// );
///
/// let mut encoded = Vec::new();
/// value.encode(&mut encoded)?;
/// let mut input = encoded.as_slice();
/// assert_eq!(ChatType::decode(&mut input)?, value);
/// assert!(input.is_empty());
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ChatType {
    /// Decoration used for the normal chat presentation.
    pub chat: ChatDecoration,
    /// Decoration used for narration.
    pub narration: ChatDecoration,
}

impl ChatType {
    /// Creates a direct chat type from its chat and narration decorations.
    #[must_use]
    pub const fn new(chat: ChatDecoration, narration: ChatDecoration) -> Self {
        Self { chat, narration }
    }
}

impl TypeCodec for ChatType {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), CodecError> {
        self.chat
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ChatType))?;
        self.narration
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::ChatType))
    }

    fn decode(reader: &mut impl std::io::Read) -> Result<Self, CodecError> {
        let chat = ChatDecoration::decode(reader)
            .map_err(|error| error.with_context(CodecKind::ChatType))?;
        let narration = ChatDecoration::decode(reader)
            .map_err(|error| error.with_context(CodecKind::ChatType))?;
        Ok(Self::new(chat, narration))
    }
}
