mod bytes;
mod maybe;
mod prefix;
mod string;
mod traits;

pub(crate) mod pattern;

use crate::fmt;

pub use self::bytes::Bytes;
pub use self::maybe::MaybeString;
pub use self::pattern::Pattern;
pub use self::string::String;
pub use self::traits::Input;

pub(crate) use self::prefix::Prefix;
pub(crate) use self::traits::{BytesLength, IntoInput, Private, PrivateExt};

/// Creates a new `Input` from a byte or string slice.
///
/// It is recommended to use this directly from the crate as `dangerous::input()`,
/// not as an import via `use` as shown below, as you lose the discoverability.
///
/// ```
/// use dangerous::input; // bad
///
/// dangerous::input(b"hello"); // do this instead
/// ```
#[inline(always)]
pub fn input<'i, I>(input: I) -> I::Input
where
    I: IntoInput<'i>,
{
    input.into_input()
}

///////////////////////////////////////////////////////////////////////////////
// Bound

/// Indication of whether [`Input`] will change in futher passes.
///
/// Used for retry functionality if enabled.
#[must_use]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Bound {
    /// Both sides of the [`Input`](crate::Input) may change in further passes.
    None,
    /// The start of the [`Input`](crate::Input) in further passes will not change.
    ///
    /// The end of the [`Input`](crate::Input) may however change in further passes.
    Start,
    /// Both sides of the [`Input`](crate::Input) in further passes will not change.
    Both,
}

impl Bound {
    #[inline(always)]
    pub(crate) fn force_close() -> Self {
        Bound::Both
    }

    /// An end is opened when it is detected a `take_consumed` reader could have
    /// continued.
    #[inline(always)]
    #[cfg(feature = "retry")]
    pub(crate) fn open_end(self) -> Self {
        match self {
            // If at least the start is bound make sure the end is unbound.
            Bound::Both | Bound::Start => Bound::Start,
            // If the start is unbound both sides of the input are unbound.
            Bound::None => Bound::None,
        }
    }

    /// An end is closed when a known length of input is sucessfully taken.
    #[inline(always)]
    pub(crate) fn close_end(self) -> Self {
        // We don't care if the input has no bounds. The only place input with
        // no bounds can originate is when a reader has reached the end of input
        // and could have consumed more. In other words - input with no bounds
        // is always empty. A length of zero taken from input with no bounds
        // will always succeed but the first half will have both sides bound to
        // prevent deadlocks.
        let _ = self;
        Bound::force_close()
    }

    #[inline(always)]
    pub(crate) fn for_end(self) -> Self {
        match self {
            // If both sides are bounded nothing will change.
            Bound::Both => Bound::Both,
            // As we have skipped to the end without checking, we don't know
            // where the start is, perhaps the true end is not known yet!
            Bound::Start | Bound::None => Bound::None,
        }
    }
}

impl fmt::Debug for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::None => "None",
            Self::Start => "Start",
            Self::Both => "Both",
        };
        f.write_str(s)
    }
}
