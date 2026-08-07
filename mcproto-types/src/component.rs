//! Data model for Minecraft text components.
//!
//! The types in this module represent the component schema shared by JSON and
//! NBT text component encodings.

use std::{collections::BTreeMap, fmt, num::NonZeroI32};

use fastnbt::Value as NbtValue;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as _, MapAccess, SeqAccess, Visitor, value::MapAccessDeserializer},
    ser::{SerializeMap, SerializeSeq},
};

/// Java Edition release whose text component schema is represented here.
pub const TEXT_COMPONENT_FORMAT_VERSION: &str = "26.1";

/// The Java Edition text component schema documented for the current protocol.
///
/// `V` is the wire format's deliberately dynamic value type. Use
/// [`NbtComponent`] for NBT and [`JsonComponent`] for JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum Component<V> {
    /// The string shorthand for a plain-text component.
    Text(String),
    /// The non-empty list shorthand.
    Sequence(ComponentSequence<V>),
    /// A full component object.
    Object(Box<ComponentObject<V>>),
}

/// A text component whose dynamic payloads use NBT values.
pub type NbtComponent = Component<NbtValue>;
/// A text component whose dynamic payloads use JSON values.
pub type JsonComponent = Component<serde_json::Value>;

impl<V> Component<V> {
    /// Creates the string shorthand for a plain-text component.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Creates a full component object with the supplied content.
    ///
    /// The new object has an empty [`Style`] and no extra components.
    pub fn object(content: Content<V>) -> Self {
        Self::Object(Box::new(ComponentObject::new(content)))
    }

    /// Creates a non-empty component sequence from its first element and the
    /// remaining elements.
    pub fn sequence(first: Component<V>, rest: impl IntoIterator<Item = Component<V>>) -> Self {
        Self::Sequence(ComponentSequence::new(first, rest))
    }

    pub(crate) fn validate_depth(&self, max_depth: usize) -> Result<(), ComponentDepthError> {
        let mut pending = vec![(self, 1_usize)];
        while let Some((component, depth)) = pending.pop() {
            if depth > max_depth {
                return Err(ComponentDepthError { max_depth });
            }
            let next = depth + 1;
            match component {
                Self::Text(_) => {}
                Self::Sequence(sequence) => {
                    pending.extend(sequence.iter().map(|child| (child, next)));
                }
                Self::Object(object) => {
                    pending.extend(object.extra.iter().map(|child| (child, next)));
                    match &object.content {
                        Content::Translatable { with, .. } => {
                            pending.extend(with.iter().map(|child| (child, next)));
                        }
                        Content::Selector { separator, .. } | Content::Nbt { separator, .. } => {
                            if let Some(separator) = separator {
                                pending.push((separator, next));
                            }
                        }
                        _ => {}
                    }
                    if let Some(hover) = &object.style.hover_event {
                        match hover {
                            HoverEvent::ShowText { value } => pending.push((value, next)),
                            HoverEvent::ShowEntity { name, .. } => {
                                if let Some(name) = name {
                                    pending.push((name, next));
                                }
                            }
                            HoverEvent::ShowItem { .. } => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Component<serde_json::Value> {
    pub(crate) fn validate_dynamic_depth(
        &self,
        max_depth: usize,
    ) -> Result<(), ComponentDepthError> {
        validate_dynamic_values(self, |root| {
            let mut pending = vec![(root, 1_usize)];
            while let Some((value, depth)) = pending.pop() {
                if depth > max_depth {
                    return Err(ComponentDepthError { max_depth });
                }
                let next = depth + 1;
                match value {
                    serde_json::Value::Array(values) => {
                        pending.extend(values.iter().map(|value| (value, next)));
                    }
                    serde_json::Value::Object(values) => {
                        pending.extend(values.values().map(|value| (value, next)));
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }
}

impl Component<NbtValue> {
    pub(crate) fn validate_dynamic_depth(
        &self,
        max_depth: usize,
    ) -> Result<(), ComponentDepthError> {
        validate_dynamic_values(self, |root| {
            let mut pending = vec![(root, 1_usize)];
            while let Some((value, depth)) = pending.pop() {
                if depth > max_depth {
                    return Err(ComponentDepthError { max_depth });
                }
                let next = depth + 1;
                match value {
                    NbtValue::List(values) => {
                        pending.extend(values.iter().map(|value| (value, next)));
                    }
                    NbtValue::Compound(values) => {
                        pending.extend(values.values().map(|value| (value, next)));
                    }
                    _ => {}
                }
            }
            Ok(())
        })
    }
}

fn validate_dynamic_values<V, E>(
    root: &Component<V>,
    mut validate: impl FnMut(&V) -> Result<(), E>,
) -> Result<(), E> {
    let mut pending = vec![root];
    while let Some(component) = pending.pop() {
        match component {
            Component::Text(_) => {}
            Component::Sequence(sequence) => pending.extend(sequence.iter()),
            Component::Object(object) => {
                pending.extend(&object.extra);
                match &object.content {
                    Content::Translatable { with, .. } => pending.extend(with),
                    Content::Selector { separator, .. } | Content::Nbt { separator, .. } => {
                        if let Some(separator) = separator {
                            pending.push(separator);
                        }
                    }
                    _ => {}
                }
                if let Some(click) = &object.style.click_event {
                    match click {
                        ClickEvent::ShowDialog {
                            dialog: DialogReference::Inline(values),
                        } => {
                            for value in values.values() {
                                validate(value)?;
                            }
                        }
                        ClickEvent::Custom {
                            payload: Some(value),
                            ..
                        } => validate(value)?,
                        _ => {}
                    }
                }
                if let Some(hover) = &object.style.hover_event {
                    match hover {
                        HoverEvent::ShowText { value } => pending.push(value),
                        HoverEvent::ShowItem { components, .. } => {
                            for value in components.values() {
                                validate(value)?;
                            }
                        }
                        HoverEvent::ShowEntity { name, .. } => {
                            if let Some(name) = name {
                                pending.push(name);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

impl<V> Default for Component<V> {
    fn default() -> Self {
        Self::text("")
    }
}

impl<V> From<String> for Component<V> {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl<V> From<&str> for Component<V> {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl<V: Serialize> Serialize for Component<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Text(text) => serializer.serialize_str(text),
            Self::Sequence(sequence) => sequence.serialize(serializer),
            Self::Object(object) => object.serialize(serializer),
        }
    }
}

impl<'de, V> Deserialize<'de> for Component<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ComponentVisitor<V>(std::marker::PhantomData<V>);

        impl<'de, V> Visitor<'de> for ComponentVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = Component<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a text component string, non-empty list, or object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Component::Text(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Component::Text(value))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut components =
                    Vec::with_capacity(sequence.size_hint().unwrap_or(0).min(1024));
                while let Some(component) = sequence.next_element()? {
                    components.push(component);
                }
                ComponentSequence::try_from(components)
                    .map(Component::Sequence)
                    .map_err(A::Error::custom)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                ComponentObject::deserialize(MapAccessDeserializer::new(map))
                    .map(Box::new)
                    .map(Component::Object)
            }
        }

        deserializer.deserialize_any(ComponentVisitor(std::marker::PhantomData))
    }
}

/// A non-empty list shorthand for a sequence of text components.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentSequence<V> {
    first: Box<Component<V>>,
    rest: Vec<Component<V>>,
}

impl<V> ComponentSequence<V> {
    /// Creates a sequence from its required first component and any remaining
    /// components.
    pub fn new(first: Component<V>, rest: impl IntoIterator<Item = Component<V>>) -> Self {
        Self {
            first: Box::new(first),
            rest: rest.into_iter().collect(),
        }
    }

    /// Returns the first component in the sequence.
    pub fn first(&self) -> &Component<V> {
        &self.first
    }

    /// Returns all components after the first one.
    pub fn rest(&self) -> &[Component<V>] {
        &self.rest
    }

    /// Iterates over every component in order, including the first one.
    pub fn iter(&self) -> impl Iterator<Item = &Component<V>> {
        std::iter::once(self.first.as_ref()).chain(&self.rest)
    }
}

impl<V> TryFrom<Vec<Component<V>>> for ComponentSequence<V> {
    type Error = EmptyComponentSequence;

    fn try_from(mut value: Vec<Component<V>>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EmptyComponentSequence);
        }
        let rest = value.split_off(1);
        let first = value.pop().ok_or(EmptyComponentSequence)?;
        Ok(Self::new(first, rest))
    }
}

impl<V: Serialize> Serialize for ComponentSequence<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(1 + self.rest.len()))?;
        for component in self.iter() {
            sequence.serialize_element(component)?;
        }
        sequence.end()
    }
}

/// Error returned when an empty list is converted into a [`ComponentSequence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyComponentSequence;

impl fmt::Display for EmptyComponentSequence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a text component sequence cannot be empty")
    }
}

impl std::error::Error for EmptyComponentSequence {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ComponentDepthError {
    pub max_depth: usize,
}

impl fmt::Display for ComponentDepthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "text component nesting exceeds {} levels",
            self.max_depth
        )
    }
}

impl std::error::Error for ComponentDepthError {}

/// A full text component object with content, style, and appended components.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentObject<V> {
    /// The content rendered by this component.
    pub content: Content<V>,
    /// Optional formatting and interaction behavior.
    pub style: Style<V>,
    /// Components appended after this component.
    pub extra: Vec<Component<V>>,
}

impl<V> ComponentObject<V> {
    /// Creates an object with the supplied content, an empty style, and no
    /// extra components.
    pub fn new(content: Content<V>) -> Self {
        Self {
            content,
            style: Style::default(),
            extra: Vec::new(),
        }
    }

    /// Creates an object containing literal text.
    pub fn text(value: impl Into<String>) -> Self {
        Self::new(Content::Text { text: value.into() })
    }
}

/// The mutually exclusive content payload of a [`ComponentObject`].
#[derive(Debug, Clone, PartialEq)]
pub enum Content<V> {
    /// Literal text.
    Text {
        /// The text to display.
        text: String,
    },
    /// Text looked up from a translation key.
    Translatable {
        /// The translation key.
        translate: String,
        /// Text used when the translation key is unavailable.
        fallback: Option<String>,
        /// Components substituted into the translated text.
        with: Vec<Component<V>>,
    },
    /// A value from a scoreboard objective.
    Score {
        /// The scoreboard holder and objective to query.
        score: Score,
    },
    /// The names selected by an entity selector.
    Selector {
        /// The entity selector expression.
        selector: String,
        /// Component placed between selected names.
        separator: Option<Box<Component<V>>>,
    },
    /// The localized name of a client key binding.
    Keybind {
        /// The key-binding identifier.
        keybind: String,
    },
    /// Values read from an NBT path.
    Nbt {
        /// The NBT path to evaluate.
        nbt: String,
        /// The entity, block, or storage source to query.
        target: NbtTarget,
        /// How extracted values are rendered.
        display: NbtDisplay,
        /// Component placed between multiple extracted values.
        separator: Option<Box<Component<V>>>,
    },
    /// A rendered object such as an atlas sprite or player head.
    Object {
        /// The object-specific rendering data.
        object: ObjectContent,
        /// Fallback text used when the object cannot be rendered.
        ///
        /// Added by Java Edition 26.1.
        fallback: Option<String>,
    },
}

/// A scoreboard holder and objective referenced by score content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    /// The score holder name or selector.
    pub name: String,
    /// The scoreboard objective name.
    pub objective: String,
}

/// The source queried by an NBT component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtTarget {
    /// An entity selected by the contained selector.
    Entity(String),
    /// A block at the contained position expression.
    Block(String),
    /// A command storage entry identified by its resource location.
    Storage(ResourceLocation),
}

/// Controls how values extracted from NBT are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NbtDisplay {
    /// Uses the default styled representation.
    #[default]
    Styled,
    /// Displays the extracted value as plain text.
    Plain,
    /// Interprets the extracted value as a serialized text component.
    Interpret,
}

/// The object rendered by object content.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectContent {
    /// A sprite from a texture atlas.
    Atlas {
        /// The atlas containing the sprite, or the protocol default.
        atlas: Option<ResourceLocation>,
        /// The sprite to render.
        sprite: ResourceLocation,
    },
    /// A player head derived from profile data.
    Player {
        /// The player name or complete profile.
        player: PlayerProfile,
        /// Whether to render the player's hat layer.
        hat: Option<bool>,
    },
}

/// The shorthand or full representation of a player profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlayerProfile {
    /// A profile resolved from a player name.
    Name(PlayerName),
    /// Explicit profile data.
    Profile(Profile),
}

/// Player profile data used to render a player object.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Profile {
    /// The player's validated account name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<PlayerName>,
    /// The player's UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    /// Signed or unsigned profile properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<ProfileProperty>,
    /// The player's skin texture resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<ResourceLocation>,
    /// The player's cape texture resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cape: Option<ResourceLocation>,
    /// The player's elytra texture resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elytra: Option<ResourceLocation>,
    /// The geometry model used for the player skin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<PlayerModel>,
}

/// A property attached to a player profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileProperty {
    /// The kind of profile property.
    pub name: ProfilePropertyName,
    /// The encoded property value.
    pub value: String,
    /// The optional signature authenticating the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// A supported player profile property name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfilePropertyName {
    /// Player skin and related texture data.
    Textures,
}

/// The geometry used to render a player skin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerModel {
    /// The standard model with wide arms.
    Wide,
    /// The slim model with narrow arms.
    Slim,
}

/// Optional formatting and interaction properties for a component.
#[derive(Debug, Clone, PartialEq)]
pub struct Style<V> {
    /// The text color.
    pub color: Option<TextColor>,
    /// The font resource used to render text.
    pub font: Option<ResourceLocation>,
    /// Whether text is rendered in bold.
    pub bold: Option<bool>,
    /// Whether text is rendered in italics.
    pub italic: Option<bool>,
    /// Whether text is underlined.
    pub underlined: Option<bool>,
    /// Whether text has a strikethrough line.
    pub strikethrough: Option<bool>,
    /// Whether text characters are continuously obfuscated.
    pub obfuscated: Option<bool>,
    /// The text shadow color.
    pub shadow_color: Option<ShadowColor>,
    /// Text inserted into chat when the component is shift-clicked.
    pub insertion: Option<String>,
    /// The action performed when the component is clicked.
    pub click_event: Option<ClickEvent<V>>,
    /// The content shown when the component is hovered.
    pub hover_event: Option<HoverEvent<V>>,
}

impl<V> Default for Style<V> {
    fn default() -> Self {
        Self {
            color: None,
            font: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            shadow_color: None,
            insertion: None,
            click_event: None,
            hover_event: None,
        }
    }
}

impl<V> Style<V> {
    fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.font.is_none()
            && self.bold.is_none()
            && self.italic.is_none()
            && self.underlined.is_none()
            && self.strikethrough.is_none()
            && self.obfuscated.is_none()
            && self.shadow_color.is_none()
            && self.insertion.is_none()
            && self.click_event.is_none()
            && self.hover_event.is_none()
    }
}

/// An action performed when a styled component is clicked.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "action",
    rename_all = "snake_case",
    bound(serialize = "V: Serialize")
)]
pub enum ClickEvent<V> {
    /// Opens an HTTP or HTTPS URL.
    OpenUrl {
        /// The URL to open.
        url: HttpUrl,
    },
    /// Opens a local file path on the client.
    OpenFile {
        /// The file path to open.
        path: String,
    },
    /// Runs a command.
    RunCommand {
        /// The command to run.
        command: CommandString,
    },
    /// Places a command into the client's chat input.
    SuggestCommand {
        /// The command to suggest.
        command: CommandString,
    },
    /// Changes the current book page.
    ChangePage {
        /// The one-based page number.
        page: PositiveI32,
    },
    /// Copies text to the system clipboard.
    CopyToClipboard {
        /// The text to copy.
        value: String,
    },
    /// Opens a dialog.
    ShowDialog {
        /// The dialog identifier or inline definition.
        dialog: DialogReference<V>,
    },
    /// Performs a custom action identified by a resource location.
    Custom {
        /// The custom action identifier.
        id: ResourceLocation,
        /// An optional wire-format-specific action payload.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<V>,
    },
}

/// A reference to a registered dialog or an inline dialog definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    untagged,
    bound(serialize = "V: Serialize", deserialize = "V: Deserialize<'de>")
)]
pub enum DialogReference<V> {
    /// The resource location of a registered dialog.
    Id(ResourceLocation),
    /// An inline dialog represented by wire-format-specific values.
    Inline(BTreeMap<String, V>),
}

/// Content shown when a styled component is hovered.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "action",
    rename_all = "snake_case",
    bound(serialize = "V: Serialize")
)]
pub enum HoverEvent<V> {
    /// Shows another text component.
    ShowText {
        /// The component displayed in the tooltip.
        value: Box<Component<V>>,
    },
    /// Shows an item stack.
    ShowItem {
        /// The item type.
        id: ResourceLocation,
        /// The optional stack size.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        count: Option<i32>,
        /// Item data components keyed by resource location.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        components: BTreeMap<ResourceLocation, V>,
    },
    /// Shows entity information.
    ShowEntity {
        /// The optional entity name displayed in the tooltip.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<Box<Component<V>>>,
        /// The entity type.
        id: ResourceLocation,
        /// The entity UUID.
        uuid: Uuid,
    },
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "V: Deserialize<'de>"))]
struct RawClickEvent<V> {
    action: ClickAction,
    #[serde(default)]
    url: Option<HttpUrl>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    command: Option<CommandString>,
    #[serde(default)]
    page: Option<PositiveI32>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    dialog: Option<DialogReference<V>>,
    #[serde(default)]
    id: Option<ResourceLocation>,
    #[serde(default)]
    payload: Option<V>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClickAction {
    OpenUrl,
    OpenFile,
    RunCommand,
    SuggestCommand,
    ChangePage,
    CopyToClipboard,
    ShowDialog,
    Custom,
}

impl<'de, V> Deserialize<'de> for ClickEvent<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawClickEvent::deserialize(deserializer)?;
        let missing = || D::Error::custom("click event is missing its action payload");
        match raw.action {
            ClickAction::OpenUrl => raw.url.map(|url| Self::OpenUrl { url }).ok_or_else(missing),
            ClickAction::OpenFile => raw
                .path
                .map(|path| Self::OpenFile { path })
                .ok_or_else(missing),
            ClickAction::RunCommand => raw
                .command
                .map(|command| Self::RunCommand { command })
                .ok_or_else(missing),
            ClickAction::SuggestCommand => raw
                .command
                .map(|command| Self::SuggestCommand { command })
                .ok_or_else(missing),
            ClickAction::ChangePage => raw
                .page
                .map(|page| Self::ChangePage { page })
                .ok_or_else(missing),
            ClickAction::CopyToClipboard => raw
                .value
                .map(|value| Self::CopyToClipboard { value })
                .ok_or_else(missing),
            ClickAction::ShowDialog => raw
                .dialog
                .map(|dialog| Self::ShowDialog { dialog })
                .ok_or_else(missing),
            ClickAction::Custom => raw
                .id
                .map(|id| Self::Custom {
                    id,
                    payload: raw.payload,
                })
                .ok_or_else(missing),
        }
    }
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "V: Deserialize<'de>"))]
struct RawHoverEvent<V> {
    action: HoverAction,
    #[serde(default)]
    value: Option<Box<Component<V>>>,
    #[serde(default)]
    id: Option<ResourceLocation>,
    #[serde(default)]
    count: Option<i32>,
    #[serde(default)]
    components: BTreeMap<ResourceLocation, V>,
    #[serde(default)]
    name: Option<Box<Component<V>>>,
    #[serde(default)]
    uuid: Option<Uuid>,
}

#[derive(Deserialize)]
enum HoverAction {
    #[serde(rename = "show_text")]
    Text,
    #[serde(rename = "show_item")]
    Item,
    #[serde(rename = "show_entity")]
    Entity,
}

impl<'de, V> Deserialize<'de> for HoverEvent<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawHoverEvent::deserialize(deserializer)?;
        let missing = || D::Error::custom("hover event is missing its action payload");
        match raw.action {
            HoverAction::Text => raw
                .value
                .map(|value| Self::ShowText { value })
                .ok_or_else(missing),
            HoverAction::Item => raw
                .id
                .map(|id| Self::ShowItem {
                    id,
                    count: raw.count,
                    components: raw.components,
                })
                .ok_or_else(missing),
            HoverAction::Entity => match (raw.id, raw.uuid) {
                (Some(id), Some(uuid)) => Ok(Self::ShowEntity {
                    name: raw.name,
                    id,
                    uuid,
                }),
                _ => Err(missing()),
            },
        }
    }
}

/// A validated Minecraft resource location.
///
/// Namespaces permit lowercase ASCII letters, digits, `_`, `.`, and `-`;
/// paths additionally permit `/`. The namespace may be omitted on input.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceLocation(String);

impl ResourceLocation {
    /// Validates and stores a resource location.
    ///
    /// Returns [`InvalidResourceLocation`] when the namespace or path contains
    /// unsupported characters or is empty.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidResourceLocation> {
        let value = value.into();
        validate_resource_location(&value)?;
        Ok(Self(value))
    }

    /// Returns the resource location exactly as it was supplied.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for ResourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Error returned when a string is not a valid [`ResourceLocation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidResourceLocation;

impl fmt::Display for InvalidResourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid Minecraft resource location")
    }
}

impl std::error::Error for InvalidResourceLocation {}

fn validate_resource_location(value: &str) -> Result<(), InvalidResourceLocation> {
    if crate::basic::is_valid_identifier(value) {
        Ok(())
    } else {
        Err(InvalidResourceLocation)
    }
}

/// A validated absolute HTTP or HTTPS URL used by an `open_url` click event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HttpUrl(String);

impl HttpUrl {
    /// Validates and stores an absolute HTTP or HTTPS URL with a host.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidHttpUrl> {
        let value = value.into();
        let parsed = url::Url::parse(&value).map_err(|_| InvalidHttpUrl)?;
        if matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some() {
            Ok(Self(value))
        } else {
            Err(InvalidHttpUrl)
        }
    }

    /// Returns the original URL string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for HttpUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for HttpUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Error returned when an `open_url` value is not an absolute HTTP or HTTPS URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidHttpUrl;

impl fmt::Display for InvalidHttpUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("open_url requires an absolute HTTP or HTTPS URL")
    }
}

impl std::error::Error for InvalidHttpUrl {}

/// A command string validated for use in a text component click event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandString(String);

impl CommandString {
    /// Validates and stores a command string.
    ///
    /// Minecraft control characters, DEL, and the legacy section sign are
    /// rejected.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCommandString> {
        let value = value.into();
        if value
            .chars()
            .all(|character| character >= ' ' && character != '\u{7f}' && character != '\u{a7}')
        {
            Ok(Self(value))
        } else {
            Err(InvalidCommandString)
        }
    }

    /// Returns the validated command string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for CommandString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CommandString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Error returned when a command contains a character forbidden by Minecraft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCommandString;

impl fmt::Display for InvalidCommandString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("command contains a character forbidden by Minecraft")
    }
}

impl std::error::Error for InvalidCommandString {}

/// A validated Java Edition player name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerName(String);

impl PlayerName {
    /// Validates and stores a player name containing 1 to 16 ASCII letters,
    /// digits, or underscores.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidPlayerName> {
        let value = value.into();
        if !value.is_empty()
            && value.len() <= 16
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            Ok(Self(value))
        } else {
            Err(InvalidPlayerName)
        }
    }

    /// Returns the validated player name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for PlayerName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PlayerName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// Error returned when a string is not a valid [`PlayerName`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPlayerName;

impl fmt::Display for InvalidPlayerName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("player name must contain 1-16 ASCII letters, digits, or underscores")
    }
}

impl std::error::Error for InvalidPlayerName {}

/// A strictly positive signed 32-bit integer.
///
/// Text components use this type for one-based book page numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PositiveI32(NonZeroI32);

impl PositiveI32 {
    /// Creates a positive integer, returning `None` for zero or negative input.
    pub fn new(value: i32) -> Option<Self> {
        NonZeroI32::new(value)
            .filter(|value| value.get() > 0)
            .map(Self)
    }

    /// Returns the contained positive integer.
    pub fn get(self) -> i32 {
        self.0.get()
    }
}

impl<'de> Deserialize<'de> for PositiveI32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Self::new(value).ok_or_else(|| D::Error::custom("page must be a positive integer"))
    }
}

/// A 128-bit universally unique identifier used by component profile data.
///
/// Parsing and formatting are delegated to the [`uuid`] crate; the wrapper
/// exists to support the custom serde representations used by text components
/// (a string, a four-integer list, or an NBT int array).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
    /// Creates a UUID from its 16 bytes in network order.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(uuid::Uuid::from_bytes(bytes))
    }

    /// Returns the UUID as 16 bytes in network order.
    pub fn into_bytes(self) -> [u8; 16] {
        self.0.into_bytes()
    }

    /// Parses a compact or hyphenated hexadecimal UUID string.
    pub fn parse(value: &str) -> Result<Self, InvalidUuid> {
        uuid::Uuid::try_parse(value)
            .map(Self)
            .map_err(|_| InvalidUuid)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UuidVisitor;

        impl<'de> Visitor<'de> for UuidVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a UUID string, four-integer list, or NBT int array")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Uuid::parse(value).map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = [0_i32; 4];
                for (index, value) in values.iter_mut().enumerate() {
                    *value = sequence
                        .next_element()?
                        .ok_or_else(|| A::Error::invalid_length(index, &self))?;
                }
                if sequence.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(A::Error::invalid_length(5, &self));
                }
                uuid_from_ints(&values).map_err(A::Error::custom)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let value = fastnbt::IntArray::deserialize(MapAccessDeserializer::new(map))?;
                uuid_from_ints(value.as_ref()).map_err(A::Error::custom)
            }
        }

        deserializer.deserialize_any(UuidVisitor)
    }
}

fn uuid_from_ints(value: &[i32]) -> Result<Uuid, InvalidUuid> {
    let value: [i32; 4] = value.try_into().map_err(|_| InvalidUuid)?;
    let mut bytes = [0_u8; 16];
    for (chunk, integer) in bytes.chunks_exact_mut(4).zip(value) {
        chunk.copy_from_slice(&integer.to_be_bytes());
    }
    Ok(Uuid::from_bytes(bytes))
}

/// Error returned when a UUID representation is malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidUuid;

impl fmt::Display for InvalidUuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid UUID")
    }
}

impl std::error::Error for InvalidUuid {}

/// A named Minecraft color or an explicit 24-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextColor {
    /// One of Minecraft's predefined named colors.
    Named(NamedColor),
    /// An explicit red, green, and blue color.
    Rgb(RgbColor),
}

impl TextColor {
    /// Creates an RGB text color from a 24-bit integer.
    pub const fn rgb(value: u32) -> Result<Self, InvalidRgbColor> {
        match RgbColor::new(value) {
            Ok(value) => Ok(Self::Rgb(value)),
            Err(error) => Err(error),
        }
    }
}

/// A 24-bit red, green, and blue color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RgbColor(u32);

impl RgbColor {
    /// The largest value that fits in an RGB color.
    pub const MAX: u32 = 0x00ff_ffff;

    /// Creates a color from a 24-bit `0xRRGGBB` integer.
    pub const fn new(value: u32) -> Result<Self, InvalidRgbColor> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(InvalidRgbColor)
        }
    }

    /// Creates a color from its red, green, and blue channels.
    pub const fn from_channels(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// Returns the color as a 24-bit `0xRRGGBB` integer.
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Returns the red, green, and blue channels in that order.
    pub const fn channels(self) -> [u8; 3] {
        [(self.0 >> 16) as u8, (self.0 >> 8) as u8, self.0 as u8]
    }
}

impl From<RgbColor> for TextColor {
    fn from(value: RgbColor) -> Self {
        Self::Rgb(value)
    }
}

/// Error returned when an RGB value does not fit in 24 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRgbColor;

impl fmt::Display for InvalidRgbColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RGB color must fit in 24 bits")
    }
}

impl std::error::Error for InvalidRgbColor {}

/// A predefined Minecraft text color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    /// Black (`#000000`).
    Black,
    /// Dark blue (`#0000AA`).
    DarkBlue,
    /// Dark green (`#00AA00`).
    DarkGreen,
    /// Dark aqua (`#00AAAA`).
    DarkAqua,
    /// Dark red (`#AA0000`).
    DarkRed,
    /// Dark purple (`#AA00AA`).
    DarkPurple,
    /// Gold (`#FFAA00`).
    Gold,
    /// Gray (`#AAAAAA`).
    Gray,
    /// Dark gray (`#555555`).
    DarkGray,
    /// Blue (`#5555FF`).
    Blue,
    /// Green (`#55FF55`).
    Green,
    /// Aqua (`#55FFFF`).
    Aqua,
    /// Red (`#FF5555`).
    Red,
    /// Light purple (`#FF55FF`).
    LightPurple,
    /// Yellow (`#FFFF55`).
    Yellow,
    /// White (`#FFFFFF`).
    White,
}

impl Serialize for TextColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Named(color) => color.serialize(serializer),
            Self::Rgb(rgb) => serializer.serialize_str(&format!("#{:06x}", rgb.value())),
        }
    }
}

impl<'de> Deserialize<'de> for TextColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Some(rgb) = value.strip_prefix('#')
            && rgb.len() == 6
        {
            return u32::from_str_radix(rgb, 16)
                .map_err(D::Error::custom)
                .and_then(|value| {
                    RgbColor::new(value)
                        .map(Self::Rgb)
                        .map_err(D::Error::custom)
                });
        }
        serde_json::from_value::<NamedColor>(serde_json::Value::String(value))
            .map(Self::Named)
            .map_err(D::Error::custom)
    }
}

/// A text shadow color stored as a packed 32-bit ARGB value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ShadowColor(i32);

impl ShadowColor {
    /// Creates a shadow color from a packed ARGB value.
    pub fn from_argb(argb: i32) -> Self {
        Self(argb)
    }

    /// Returns the packed ARGB value.
    pub fn argb(self) -> i32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ShadowColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Argb(i32),
            Rgba([f32; 4]),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Argb(argb) => Ok(Self(argb)),
            Repr::Rgba(rgba) => {
                if rgba
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
                {
                    return Err(D::Error::custom(
                        "shadow color channels must be between 0 and 1",
                    ));
                }
                let channel = |value: f32| (value * 255.0).round() as u32;
                let [red, green, blue, alpha] = rgba.map(channel);
                Ok(Self(
                    ((alpha << 24) | (red << 16) | (green << 8) | blue) as i32,
                ))
            }
        }
    }
}

impl<V: Serialize> Serialize for ComponentObject<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        serialize_content(&mut map, &self.content)?;
        serialize_style(&mut map, &self.style)?;
        if !self.extra.is_empty() {
            map.serialize_entry("extra", &self.extra)?;
        }
        map.end()
    }
}

fn serialize_content<M, V>(map: &mut M, content: &Content<V>) -> Result<(), M::Error>
where
    M: SerializeMap,
    V: Serialize,
{
    match content {
        Content::Text { text } => {
            map.serialize_entry("type", "text")?;
            map.serialize_entry("text", text)?;
        }
        Content::Translatable {
            translate,
            fallback,
            with,
        } => {
            map.serialize_entry("type", "translatable")?;
            map.serialize_entry("translate", translate)?;
            if let Some(fallback) = fallback {
                map.serialize_entry("fallback", fallback)?;
            }
            if !with.is_empty() {
                map.serialize_entry("with", with)?;
            }
        }
        Content::Score { score } => {
            map.serialize_entry("type", "score")?;
            map.serialize_entry("score", score)?;
        }
        Content::Selector {
            selector,
            separator,
        } => {
            map.serialize_entry("type", "selector")?;
            map.serialize_entry("selector", selector)?;
            if let Some(separator) = separator {
                map.serialize_entry("separator", separator)?;
            }
        }
        Content::Keybind { keybind } => {
            map.serialize_entry("type", "keybind")?;
            map.serialize_entry("keybind", keybind)?;
        }
        Content::Nbt {
            nbt,
            target,
            display,
            separator,
        } => {
            map.serialize_entry("type", "nbt")?;
            map.serialize_entry("nbt", nbt)?;
            match target {
                NbtTarget::Entity(entity) => {
                    map.serialize_entry("source", "entity")?;
                    map.serialize_entry("entity", entity)?;
                }
                NbtTarget::Block(block) => {
                    map.serialize_entry("source", "block")?;
                    map.serialize_entry("block", block)?;
                }
                NbtTarget::Storage(storage) => {
                    map.serialize_entry("source", "storage")?;
                    map.serialize_entry("storage", storage)?;
                }
            }
            match display {
                NbtDisplay::Styled => {}
                NbtDisplay::Plain => map.serialize_entry("plain", &true)?,
                NbtDisplay::Interpret => map.serialize_entry("interpret", &true)?,
            }
            if let Some(separator) = separator {
                map.serialize_entry("separator", separator)?;
            }
        }
        Content::Object { object, fallback } => {
            map.serialize_entry("type", "object")?;
            match object {
                ObjectContent::Atlas { atlas, sprite } => {
                    map.serialize_entry("object", "atlas")?;
                    if let Some(atlas) = atlas {
                        map.serialize_entry("atlas", atlas)?;
                    }
                    map.serialize_entry("sprite", sprite)?;
                }
                ObjectContent::Player { player, hat } => {
                    map.serialize_entry("object", "player")?;
                    map.serialize_entry("player", player)?;
                    if let Some(hat) = hat {
                        map.serialize_entry("hat", hat)?;
                    }
                }
            }
            if let Some(fallback) = fallback {
                map.serialize_entry("fallback", fallback)?;
            }
        }
    }
    Ok(())
}

fn serialize_style<M, V>(map: &mut M, style: &Style<V>) -> Result<(), M::Error>
where
    M: SerializeMap,
    V: Serialize,
{
    macro_rules! optional {
        ($field:ident) => {
            if let Some(value) = &style.$field {
                map.serialize_entry(stringify!($field), value)?;
            }
        };
    }
    optional!(color);
    optional!(font);
    optional!(bold);
    optional!(italic);
    optional!(underlined);
    optional!(strikethrough);
    optional!(obfuscated);
    optional!(shadow_color);
    optional!(insertion);
    optional!(click_event);
    optional!(hover_event);
    Ok(())
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "V: Deserialize<'de>"))]
struct RawComponent<V> {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    translate: Option<String>,
    #[serde(default)]
    fallback: Option<String>,
    #[serde(default)]
    with: Vec<Component<V>>,
    #[serde(default)]
    score: Option<Score>,
    #[serde(default)]
    selector: Option<String>,
    #[serde(default)]
    separator: Option<Box<Component<V>>>,
    #[serde(default)]
    keybind: Option<String>,
    #[serde(default)]
    nbt: Option<String>,
    #[serde(default)]
    source: Option<NbtSource>,
    #[serde(default)]
    interpret: bool,
    #[serde(default)]
    plain: bool,
    #[serde(default)]
    entity: Option<String>,
    #[serde(default)]
    block: Option<String>,
    #[serde(default)]
    storage: Option<ResourceLocation>,
    #[serde(default)]
    object: Option<String>,
    #[serde(default)]
    atlas: Option<ResourceLocation>,
    #[serde(default)]
    sprite: Option<ResourceLocation>,
    #[serde(default)]
    player: Option<PlayerProfile>,
    #[serde(default)]
    hat: Option<bool>,
    #[serde(default)]
    extra: Vec<Component<V>>,
    #[serde(default)]
    color: Option<TextColor>,
    #[serde(default)]
    font: Option<ResourceLocation>,
    #[serde(default)]
    bold: Option<bool>,
    #[serde(default)]
    italic: Option<bool>,
    #[serde(default)]
    underlined: Option<bool>,
    #[serde(default)]
    strikethrough: Option<bool>,
    #[serde(default)]
    obfuscated: Option<bool>,
    #[serde(default)]
    shadow_color: Option<ShadowColor>,
    #[serde(default)]
    insertion: Option<String>,
    #[serde(default)]
    click_event: Option<ClickEvent<V>>,
    #[serde(default)]
    hover_event: Option<HoverEvent<V>>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NbtSource {
    Entity,
    Block,
    Storage,
}

impl<'de, V> Deserialize<'de> for ComponentObject<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawComponent::deserialize(deserializer)?;
        raw.try_into().map_err(D::Error::custom)
    }
}

impl<V> TryFrom<RawComponent<V>> for ComponentObject<V> {
    type Error = InvalidComponentObject;

    fn try_from(raw: RawComponent<V>) -> Result<Self, Self::Error> {
        let selected = select_content(&raw).ok_or(InvalidComponentObject::MissingContent)?;
        let content = match selected {
            SelectedContent::Text => Content::Text {
                text: raw.text.ok_or(InvalidComponentObject::MissingContent)?,
            },
            SelectedContent::Translatable => Content::Translatable {
                translate: raw
                    .translate
                    .ok_or(InvalidComponentObject::MissingContent)?,
                fallback: raw.fallback,
                with: raw.with,
            },
            SelectedContent::Score => Content::Score {
                score: raw.score.ok_or(InvalidComponentObject::MissingContent)?,
            },
            SelectedContent::Selector => Content::Selector {
                selector: raw.selector.ok_or(InvalidComponentObject::MissingContent)?,
                separator: raw.separator,
            },
            SelectedContent::Keybind => Content::Keybind {
                keybind: raw.keybind.ok_or(InvalidComponentObject::MissingContent)?,
            },
            SelectedContent::Nbt => {
                if raw.interpret && raw.plain {
                    return Err(InvalidComponentObject::ConflictingNbtDisplay);
                }
                let target =
                    select_nbt_target(&raw).ok_or(InvalidComponentObject::MissingNbtTarget)?;
                let display = if raw.interpret {
                    NbtDisplay::Interpret
                } else if raw.plain {
                    NbtDisplay::Plain
                } else {
                    NbtDisplay::Styled
                };
                Content::Nbt {
                    nbt: raw.nbt.ok_or(InvalidComponentObject::MissingContent)?,
                    target,
                    display,
                    separator: raw.separator,
                }
            }
            SelectedContent::Object => {
                let object = match raw.object.as_deref() {
                    Some("player") => ObjectContent::Player {
                        player: raw
                            .player
                            .ok_or(InvalidComponentObject::MissingObjectField)?,
                        hat: raw.hat,
                    },
                    None | Some("atlas") => ObjectContent::Atlas {
                        atlas: raw.atlas,
                        sprite: raw
                            .sprite
                            .ok_or(InvalidComponentObject::MissingObjectField)?,
                    },
                    Some(_) => return Err(InvalidComponentObject::InvalidObjectType),
                };
                Content::Object {
                    object,
                    fallback: raw.fallback,
                }
            }
        };
        Ok(Self {
            content,
            style: Style {
                color: raw.color,
                font: raw.font,
                bold: raw.bold,
                italic: raw.italic,
                underlined: raw.underlined,
                strikethrough: raw.strikethrough,
                obfuscated: raw.obfuscated,
                shadow_color: raw.shadow_color,
                insertion: raw.insertion,
                click_event: raw.click_event,
                hover_event: raw.hover_event,
            },
            extra: raw.extra,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum SelectedContent {
    Text,
    Translatable,
    Score,
    Selector,
    Keybind,
    Nbt,
    Object,
}

fn select_content<V>(raw: &RawComponent<V>) -> Option<SelectedContent> {
    let explicit = match raw.kind.as_deref() {
        Some("text") if raw.text.is_some() => Some(SelectedContent::Text),
        Some("translatable") if raw.translate.is_some() => Some(SelectedContent::Translatable),
        Some("score") if raw.score.is_some() => Some(SelectedContent::Score),
        Some("selector") if raw.selector.is_some() => Some(SelectedContent::Selector),
        Some("keybind") if raw.keybind.is_some() => Some(SelectedContent::Keybind),
        Some("nbt") if raw.nbt.is_some() && select_nbt_target(raw).is_some() => {
            Some(SelectedContent::Nbt)
        }
        Some("object") if object_fields_are_valid(raw) => Some(SelectedContent::Object),
        _ => None,
    };
    explicit.or_else(|| {
        if raw.text.is_some() {
            Some(SelectedContent::Text)
        } else if raw.translate.is_some() {
            Some(SelectedContent::Translatable)
        } else if raw.score.is_some() {
            Some(SelectedContent::Score)
        } else if raw.selector.is_some() {
            Some(SelectedContent::Selector)
        } else if raw.keybind.is_some() {
            Some(SelectedContent::Keybind)
        } else if raw.nbt.is_some() && select_nbt_target(raw).is_some() {
            Some(SelectedContent::Nbt)
        } else if object_fields_are_valid(raw) {
            Some(SelectedContent::Object)
        } else {
            None
        }
    })
}

fn object_fields_are_valid<V>(raw: &RawComponent<V>) -> bool {
    match raw.object.as_deref() {
        Some("player") => raw.player.is_some(),
        None | Some("atlas") => raw.sprite.is_some(),
        Some(_) => false,
    }
}

fn select_nbt_target<V>(raw: &RawComponent<V>) -> Option<NbtTarget> {
    match raw.source {
        Some(NbtSource::Entity) => raw.entity.clone().map(NbtTarget::Entity),
        Some(NbtSource::Block) => raw.block.clone().map(NbtTarget::Block),
        Some(NbtSource::Storage) => raw.storage.clone().map(NbtTarget::Storage),
        None => raw
            .entity
            .clone()
            .map(NbtTarget::Entity)
            .or_else(|| raw.block.clone().map(NbtTarget::Block))
            .or_else(|| raw.storage.clone().map(NbtTarget::Storage)),
    }
}

/// Describes why a serialized component object does not match the component
/// schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidComponentObject {
    /// No recognized content field is present.
    MissingContent,
    /// NBT content does not identify an entity, block, or storage source.
    MissingNbtTarget,
    /// NBT content requests both plain and interpreted display modes.
    ConflictingNbtDisplay,
    /// Object content is missing a field required by its object type.
    MissingObjectField,
    /// The object type is not recognized.
    InvalidObjectType,
}

impl fmt::Display for InvalidComponentObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingContent => "text component object has no valid content",
            Self::MissingNbtTarget => "NBT text component has no matching source",
            Self::ConflictingNbtDisplay => "NBT text component cannot be plain and interpreted",
            Self::MissingObjectField => "object text component is missing a required field",
            Self::InvalidObjectType => "unknown object text component type",
        })
    }
}

impl std::error::Error for InvalidComponentObject {}

impl Component<NbtValue> {
    pub(crate) fn normalized_root_for_nbt(&self) -> Self {
        let component = normalize_nbt(self.clone());
        match component {
            Component::Sequence(_) => Component::Object(Box::new(component_into_object(component))),
            Component::Object(object) => {
                let ComponentObject {
                    content,
                    style,
                    extra,
                } = *object;
                match content {
                    Content::Text { text } if style.is_empty() && extra.is_empty() => {
                        Component::Text(text)
                    }
                    content => Component::Object(Box::new(ComponentObject {
                        content,
                        style,
                        extra,
                    })),
                }
            }
            _ => component,
        }
    }
}

fn normalize_nbt(component: NbtComponent) -> NbtComponent {
    match component {
        Component::Text(_) => component,
        Component::Sequence(sequence) => {
            let ComponentSequence { first, rest } = sequence;
            let first = Component::Object(Box::new(component_into_object(normalize_nbt(*first))));
            let rest = rest
                .into_iter()
                .map(normalize_nbt)
                .map(component_into_object)
                .map(Box::new)
                .map(Component::Object);
            Component::Sequence(ComponentSequence::new(first, rest))
        }
        Component::Object(mut object) => {
            object.extra = object
                .extra
                .into_iter()
                .map(normalize_nbt)
                .map(component_into_object)
                .map(Box::new)
                .map(Component::Object)
                .collect();
            match &mut object.content {
                Content::Translatable { with, .. } => {
                    *with = std::mem::take(with)
                        .into_iter()
                        .map(normalize_nbt)
                        .map(component_into_object)
                        .map(Box::new)
                        .map(Component::Object)
                        .collect();
                }
                Content::Selector { separator, .. } | Content::Nbt { separator, .. } => {
                    if let Some(value) = separator.take() {
                        *separator = Some(Box::new(normalize_nbt(*value)));
                    }
                }
                _ => {}
            }
            if let Some(hover) = &mut object.style.hover_event {
                match hover {
                    HoverEvent::ShowText { value } => {
                        **value = normalize_nbt(std::mem::take(value.as_mut()));
                    }
                    HoverEvent::ShowEntity { name, .. } => {
                        if let Some(value) = name.take() {
                            *name = Some(Box::new(normalize_nbt(*value)));
                        }
                    }
                    HoverEvent::ShowItem { .. } => {}
                }
            }
            Component::Object(object)
        }
    }
}

fn component_into_object(component: NbtComponent) -> ComponentObject<NbtValue> {
    match component {
        Component::Text(text) => ComponentObject::text(text),
        Component::Object(object) => *object,
        Component::Sequence(sequence) => {
            let ComponentSequence { first, rest } = sequence;
            let mut first = component_into_object(*first);
            first.extra.extend(
                rest.into_iter()
                    .map(component_into_object)
                    .map(Box::new)
                    .map(Component::Object),
            );
            first
        }
    }
}
