//! Debug subscription events.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind};

use crate::TypeCodec;

use super::{DebugSubscriptionData, DebugSubscriptionType};

/// A typed debug subscription event.
///
/// The wire representation is a VarInt [`DebugSubscriptionType`] followed by
/// the payload selected by that type. The discriminator is derived from
/// [`data`](Self::data), so an event cannot contain mismatched type and data.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     DebugSubscriptionData, DebugSubscriptionEvent, EntityBlockIntersectionDebugData,
///     EntityBlockIntersectionState, TypeCodec,
/// };
///
/// let event = DebugSubscriptionEvent::new(
///     DebugSubscriptionData::EntityBlockIntersection(
///         EntityBlockIntersectionDebugData {
///             state: EntityBlockIntersectionState::InFluid,
///         },
///     ),
/// );
/// let mut encoded = Vec::new();
/// event.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x06, 0x01]);
/// assert_eq!(DebugSubscriptionEvent::decode(&mut encoded.as_slice())?, event);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// See the official [Debug Subscription Event] documentation.
///
/// [Debug Subscription Event]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Subscription_Event
#[derive(Debug, Clone, PartialEq)]
pub struct DebugSubscriptionEvent {
    /// Payload and its associated subscription type.
    pub data: DebugSubscriptionData,
}

impl DebugSubscriptionEvent {
    /// Creates an event from a typed payload.
    #[must_use]
    pub const fn new(data: DebugSubscriptionData) -> Self {
        Self { data }
    }

    /// Returns the discriminator associated with this event.
    #[must_use]
    pub const fn subscription_type(&self) -> DebugSubscriptionType {
        self.data.subscription_type()
    }
}

impl From<DebugSubscriptionData> for DebugSubscriptionEvent {
    fn from(data: DebugSubscriptionData) -> Self {
        Self::new(data)
    }
}

impl TypeCodec for DebugSubscriptionEvent {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.subscription_type()
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionEvent))?;
        self.data
            .encode_payload(writer)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionEvent))
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let subscription_type = DebugSubscriptionType::decode(reader)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionEvent))?;
        DebugSubscriptionData::decode_payload(subscription_type, reader)
            .map(Self::new)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionEvent))
    }
}
