mod bytes;
mod entry;
mod maybe;
mod string;
mod traits;

pub use self::bytes::Bytes;
pub use self::entry::input;
pub use self::maybe::MaybeString;
pub use self::string::String;
pub use self::traits::Input;

pub(crate) use self::traits::{Private, PrivateExt};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Bound {
    /// Both sides of the [`crate::Input`] may change in further passes.
    None,
    /// The start of the [`crate::Input`] in further passes will not change.
    ///
    /// The end of the [`crate::Input`] may however change in further passes.
    Start,
    /// Both sides of the [`crate::Input`] in further passes will not change.
    Both,
}

impl Bound {
    pub(crate) fn close_end(self) -> Self {
        Bound::Both
    }

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
