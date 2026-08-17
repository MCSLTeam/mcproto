//! Debug subscription updates.

use std::io::{Read, Write};

use mcproto_codec::error::{CodecError, CodecKind};

use crate::{Boolean, TypeCodec};

use super::{DebugSubscriptionData, DebugSubscriptionType};

/// A debug subscription type followed by prefixed optional matching data.
///
/// [`Absent`](Self::Absent) writes the type and a false boolean. A
/// [`Present`](Self::Present) value writes the payload's type, a true boolean,
/// and the matching payload. This representation cannot pair one subscription
/// type with another type's data.
///
/// # Examples
///
/// ```
/// use mcproto_types::{
///     DebugSubscriptionData, DebugSubscriptionType, DebugSubscriptionUpdate, TypeCodec,
/// };
///
/// let absent = DebugSubscriptionUpdate::absent(DebugSubscriptionType::Bee);
/// let mut encoded = Vec::new();
/// absent.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x01, 0x00]);
/// assert_eq!(DebugSubscriptionUpdate::decode(&mut encoded.as_slice())?, absent);
///
/// let present = DebugSubscriptionUpdate::present(
///     DebugSubscriptionData::DedicatedServerTickTime,
/// );
/// let mut encoded = Vec::new();
/// present.encode(&mut encoded)?;
/// assert_eq!(encoded, [0x00, 0x01]);
/// # Ok::<(), mcproto_codec::error::CodecError>(())
/// ```
///
/// See the official [Debug Subscription Update] documentation.
///
/// [Debug Subscription Update]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Subscription_Update
#[derive(Debug, Clone, PartialEq)]
pub enum DebugSubscriptionUpdate {
    /// The selected subscription type has no following payload.
    Absent(DebugSubscriptionType),
    /// The selected subscription type is followed by its payload.
    Present(DebugSubscriptionData),
}

impl DebugSubscriptionUpdate {
    /// Creates an update with no payload.
    #[must_use]
    pub const fn absent(subscription_type: DebugSubscriptionType) -> Self {
        Self::Absent(subscription_type)
    }

    /// Creates an update containing matching typed data.
    #[must_use]
    pub const fn present(data: DebugSubscriptionData) -> Self {
        Self::Present(data)
    }

    /// Returns the selected subscription type.
    #[must_use]
    pub const fn subscription_type(&self) -> DebugSubscriptionType {
        match self {
            Self::Absent(subscription_type) => *subscription_type,
            Self::Present(data) => data.subscription_type(),
        }
    }

    /// Returns the payload when present.
    #[must_use]
    pub const fn data(&self) -> Option<&DebugSubscriptionData> {
        match self {
            Self::Absent(_) => None,
            Self::Present(data) => Some(data),
        }
    }

    /// Returns whether this update contains a payload.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }
}

impl TypeCodec for DebugSubscriptionUpdate {
    fn encode(&self, writer: &mut impl Write) -> Result<(), CodecError> {
        self.subscription_type()
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))?;

        let data = self.data();
        Boolean(data.is_some())
            .encode(writer)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))?;
        if let Some(data) = data {
            data.encode_payload(writer)
                .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))?;
        }
        Ok(())
    }

    fn decode(reader: &mut impl Read) -> Result<Self, CodecError> {
        let subscription_type = DebugSubscriptionType::decode(reader)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))?;
        let present = Boolean::decode(reader)
            .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))?;
        if present.0 {
            DebugSubscriptionData::decode_payload(subscription_type, reader)
                .map(Self::Present)
                .map_err(|error| error.with_context(CodecKind::DebugSubscriptionUpdate))
        } else {
            Ok(Self::Absent(subscription_type))
        }
    }
}
