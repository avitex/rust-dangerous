use crate::display::byte_count;
use crate::fmt;

/// Length that was expected in an operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[must_use]
pub enum Length {
    /// A minimum length was expected.
    AtLeast(usize),
    /// An exact length was expected.
    Exactly(usize),
}

impl Length {
    /// The minimum length that was expected.
    #[must_use]
    #[inline(always)]
    pub fn min(self) -> usize {
        match self {
            Length::AtLeast(min) | Length::Exactly(min) => min,
        }
    }

    /// The maximum length that was expected, if applicable.
    #[must_use]
    #[inline(always)]
    pub fn max(self) -> Option<usize> {
        match self {
            Length::AtLeast(_) => None,
            Length::Exactly(max) => Some(max),
        }
    }
}

impl fmt::DisplayBase for Length {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match *self {
            Self::AtLeast(min) => {
                w.write_str("at least ")?;
                byte_count(w, min)
            }
            Self::Exactly(exact) => {
                w.write_str("exactly ")?;
                byte_count(w, exact)
            }
        }
    }
}
