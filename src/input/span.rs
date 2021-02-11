use core::ops::Range;
use core::ptr::NonNull;
use core::slice;

use crate::display::InputDisplay;
use crate::fmt;
use crate::input::MaybeString;

#[must_use]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    start: NonNull<u8>,
    end: NonNull<u8>,
}

impl Span {
    pub fn start(self) -> Self {
        Self {
            start: self.start,
            end: self.end,
        }
    }

    pub fn of(self, parent: &[u8]) -> Option<&[u8]> {
        if self.is_within(parent.into()) {
            unsafe { Some(slice::from_raw_parts(self.start.as_ptr(), self.len())) }
        } else {
            None
        }
    }

    /// Returns `Some(Range)` with the `start` and `end` offsets of `self`
    /// within the `parent`. `None` is returned if `self` is not within in the
    /// `parent`.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::{Input, Span};
    ///
    /// let parent = &[1, 2, 3, 4][..];
    /// let sub_range = 1..2;
    /// let sub = &parent[sub_range.clone()];
    ///
    /// assert_eq!(Span::from(sub).range_of(parent.into()), Some(sub_range))
    /// ```
    #[must_use]
    pub fn range_of(self, parent: Span) -> Option<Range<usize>> {
        if self.is_within(parent) {
            let start_offset = self.start.as_ptr() as usize - parent.start.as_ptr() as usize;
            let end_offset = self.end.as_ptr() as usize - parent.start.as_ptr() as usize;
            Some(start_offset..end_offset)
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn debug_for(self, input: MaybeString<'_>) -> DebugFor<'_> {
        DebugFor {
            bytes: input.as_dangerous_bytes(),
            str_hint: input.is_string(),
            span: self,
        }
    }

    #[inline(always)]
    pub fn non_empty(self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    #[inline(always)]
    pub fn len(self) -> usize {
        self.end.as_ptr() as usize - self.start.as_ptr() as usize
    }

    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline(always)]
    pub fn is_within(self, other: Span) -> bool {
        other.start <= self.start && other.end >= self.end
    }

    #[inline(always)]
    pub fn offset_within(self, other: Span) -> Option<usize> {
        if self.is_within(other) {
            Some(self.start.as_ptr() as usize - other.start.as_ptr() as usize)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn is_start_of(self, other: Span) -> bool {
        self.is_empty() && other.start == self.start
    }

    #[inline(always)]
    pub fn is_end_of(self, other: Span) -> bool {
        self.is_empty() && other.end == self.end
    }

    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let all = b"0123456789";
    /// let a = Span::from(&all[0..9]);
    /// let b = Span::from(&all[6..9]);
    ///
    /// assert!(a.is_overlapping_start_of(b));
    ///
    /// ```
    #[inline(always)]
    pub fn is_overlapping_start_of(self, other: Span) -> bool {
        other.start > self.start
    }

    #[inline(always)]
    pub fn is_overlapping_end_of(self, other: Span) -> bool {
        other.end < self.end
    }

    #[inline(always)]
    pub fn is_start_within(self, other: Span) -> bool {
        other.start <= self.start && other.end > self.start
    }
}

impl fmt::DisplayBase for Span {
    fn fmt(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str("(ptr: ")?;
        w.write_usize(self.start.as_ptr() as usize)?;
        w.write_str(", len: ")?;
        w.write_usize(self.len())?;
        w.write_char(')')
    }
}

impl From<&[u8]> for Span {
    #[inline(always)]
    fn from(value: &[u8]) -> Self {
        let range = value.as_ptr_range();
        unsafe {
            Self {
                start: NonNull::new_unchecked(range.start as _),
                end: NonNull::new_unchecked(range.end as _),
            }
        }
    }
}

impl From<&str> for Span {
    #[inline(always)]
    fn from(value: &str) -> Self {
        Self::from(value.as_bytes())
    }
}

#[must_use]
pub struct DebugFor<'a> {
    span: Span,
    str_hint: bool,
    bytes: &'a [u8],
}

impl<'a> fmt::Debug for DebugFor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span.of(self.bytes) {
            Some(valid) => {
                let display = InputDisplay::from_bytes(valid).with_formatter(f);
                let display = if self.str_hint {
                    display.str_hint()
                } else {
                    display
                };
                f.debug_tuple("Span").field(&display).finish()
            }
            None => fmt::Debug::fmt(&self.span, f),
        }
    }
}
