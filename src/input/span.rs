use core::ops::Range;
use core::ptr::NonNull;
use core::slice;

use crate::display::InputDisplay;
use crate::fmt;
use crate::input::{Input, MaybeString};

/// Range of [`Input`].
///
/// Spans are specific to the input chain they were created in as the range is
/// stored as raw start and end pointers.
///
/// You can create a span from either [`Input::span()`] or from a raw slice via
/// [`Span::from()`].
///
/// [`Input`]: crate::Input  
/// [`Input::span()`]: crate::Input::span()
#[must_use]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    start: NonNull<u8>,
    end: NonNull<u8>,
}

impl Span {
    /// Returns the number of bytes spanned.
    #[must_use]
    #[inline(always)]
    pub fn len(self) -> usize {
        self.end.as_ptr() as usize - self.start.as_ptr() as usize
    }

    /// Returns `true` if no bytes are spanned.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns `true` if the span is completely within the bounds of the
    /// specified parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let bytes = [0_u8; 64];
    ///
    /// // Within
    /// let parent = Span::from(&bytes[16..32]);
    /// let child = Span::from(&bytes[20..24]);
    /// assert!(child.is_within(parent));
    /// assert!(parent.is_within(parent));
    ///
    /// // Left out of bound
    /// let parent = Span::from(&bytes[16..32]);
    /// let child = Span::from(&bytes[15..24]);
    /// assert!(!child.is_within(parent));
    ///
    /// // Right out of bound
    /// let parent = Span::from(&bytes[16..32]);
    /// let child = Span::from(&bytes[20..33]);
    /// assert!(!child.is_within(parent));
    ///
    /// // Both out of bound
    /// let parent = Span::from(&bytes[16..32]);
    /// let child = Span::from(&bytes[15..33]);
    /// assert!(!child.is_within(parent));
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn is_within(self, other: Span) -> bool {
        other.start <= self.start && other.end >= self.end
    }

    /// Returns `true` if `self` points to the start of `other`, spanning no bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let bytes = &[1, 2, 3, 4][..];
    /// let span = Span::from(bytes);
    ///
    /// assert!(span.start().is_start_of(span));
    /// assert!(!span.is_start_of(span));
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn is_start_of(self, other: Span) -> bool {
        self.is_empty() && other.start == self.start
    }

    /// Returns `true` if `self` points to the end of `other`, spanning no bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let bytes = &[1, 2, 3, 4][..];
    /// let span = Span::from(bytes);
    ///
    /// assert!(span.end().is_end_of(span));
    /// assert!(!span.is_end_of(span));
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn is_end_of(self, other: Span) -> bool {
        self.is_empty() && other.end == self.end
    }

    /// Returns `true` if `self` overlaps the start of `other`.
    ///
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
    #[must_use]
    #[inline(always)]
    pub fn is_overlapping_start_of(self, other: Span) -> bool {
        other.start > self.start
    }

    /// Returns `true` if `self` overlaps the end of `other`.
    #[must_use]
    #[inline(always)]
    pub fn is_overlapping_end_of(self, other: Span) -> bool {
        other.end < self.end
    }

    /// Returns `true` if `self`'s start is within `other`.
    #[must_use]
    #[inline(always)]
    #[allow(clippy::suspicious_operation_groupings)]
    pub fn is_start_within(self, other: Span) -> bool {
        self.start >= other.start && self.start < other.end
    }

    /// Returns `true` if `self`'s end is within `other`.
    #[must_use]
    #[inline(always)]
    #[allow(clippy::suspicious_operation_groupings)]
    pub fn is_end_within(self, other: Span) -> bool {
        self.end >= other.start && self.end < other.end
    }

    /// Returns a span pointing to the start of self, spanning no bytes.
    pub fn start(self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    /// Returns a span pointing to the end of self, spanning no bytes.
    pub fn end(self) -> Self {
        Self {
            start: self.end,
            end: self.end,
        }
    }

    /// Returns the sub slice of the provided parent `self` refers to or `None`
    /// if `self` is not within the parent or does not align with start and end
    /// token boundaries.
    ///
    /// You can get a span of [`Input`], `&str` or `&[u8]`.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let parent = &[1, 2, 3, 4][..];
    /// let sub = &parent[1..2];
    /// assert_eq!(Span::from(sub).of(parent), Some(sub));
    ///
    /// let non_span = Span::from(&[1, 2, 2, 4][..]);
    /// assert_eq!(non_span.of(parent), None);
    /// ```
    #[must_use]
    pub fn of<P>(self, parent: P) -> Option<P>
    where
        P: Parent,
    {
        parent.extract(self)
    }

    /// Returns `Some(Range)` with the `start` and `end` offsets of `self`
    /// within the `parent`. `None` is returned if `self` is not within in the
    /// `parent`.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let parent = &[1, 2, 3, 4][..];
    /// let sub_range = 1..2;
    /// let sub = &parent[sub_range.clone()];
    ///
    /// assert_eq!(Span::from(sub).range_of(parent.into()), Some(sub_range))
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn range_of(self, parent: Span) -> Option<Range<usize>> {
        if self.is_within(parent) {
            let start_offset = self.start.as_ptr() as usize - parent.start.as_ptr() as usize;
            let end_offset = self.end.as_ptr() as usize - parent.start.as_ptr() as usize;
            Some(start_offset..end_offset)
        } else {
            None
        }
    }

    /// Returns `None` if the span is empty, `Some(Self)` if not.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Span;
    ///
    /// let bytes = &[0][..];
    /// assert!(Span::from(bytes).non_empty().is_some());
    ///
    /// let bytes = &[][..];
    /// assert!(Span::from(bytes).non_empty().is_none());
    /// ```
    #[must_use]
    #[inline(always)]
    pub fn non_empty(self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    /// Wraps the span with improved debugging support given the containing
    /// input.
    #[inline(always)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn debug_for(self, input: MaybeString<'_>) -> DebugFor<'_> {
        DebugFor {
            bytes: input.as_dangerous_bytes(),
            str_hint: input.is_string(),
            span: self,
        }
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
        // SAFETY: it is invalid for a slice ptr to be null.
        // See: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html
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

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field("ptr", &self.start)
            .field("len", &self.len())
            .finish()
    }
}

// SAFETY: Span can only dereference the data pointed to if the data is present
// and passed as a parent. This makes the internal pointers safe to alias across
// threads as the parent data enforces the aliasing rules.
unsafe impl Send for Span {}

// SAFETY: Span can only dereference the data pointed to if the data is present
// and passed as a parent. This makes the internal pointers safe to alias across
// threads as the parent data enforces the aliasing rules.
unsafe impl Sync for Span {}

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

pub trait Parent: Sized {
    fn extract(self, span: Span) -> Option<Self>;
}

impl Parent for &[u8] {
    #[inline(always)]
    fn extract(self, span: Span) -> Option<Self> {
        if span.is_within(self.into()) {
            // SAFETY: we have checked that the slice is valid within the
            // parent, so we can create a slice from our bounds.
            unsafe { Some(slice::from_raw_parts(span.start.as_ptr(), span.len())) }
        } else {
            None
        }
    }
}

impl Parent for &str {
    #[inline]
    fn extract(self, span: Span) -> Option<Self> {
        span.range_of(self.into()).and_then(|range| {
            if self.is_char_boundary(range.start) && self.is_char_boundary(range.end) {
                Some(&self[range])
            } else {
                None
            }
        })
    }
}

impl<'i, T> Parent for T
where
    T: Input<'i>,
{
    #[inline]
    fn extract(self, span: Span) -> Option<Self> {
        span.range_of(self.span()).and_then(|range| {
            if self.verify_token_boundary(range.start).is_ok()
                && self.verify_token_boundary(range.end).is_ok()
            {
                // SAFETY: we have checked that the range start and end are
                // valid boundary indexes within the parent, so we can split
                // with them.
                let sub = unsafe {
                    let (_, tail) = self.split_at_byte_unchecked(range.start);
                    let (sub, _) = tail.split_at_byte_unchecked(range.end - range.start);
                    sub
                };
                Some(sub)
            } else {
                None
            }
        })
    }
}
