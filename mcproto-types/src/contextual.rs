//! Context used by protocol values whose wire representation depends on their
//! enclosing packet or data structure.

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
