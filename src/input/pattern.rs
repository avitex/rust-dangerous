use crate::input::{Bytes, String};

/// A structure that can be found within an [`Input`](crate::Input).
///
/// # Safety
///
/// The implementation must returned valid indexes and lengths for splitting
/// input as these are not checked.
pub unsafe trait Pattern<I> {
    /// Returns the start byte index and byte length of the match if found.
    fn find(self, input: &I) -> Option<(usize, usize)>;

    // TODO
    // Returns the start byte index of where the pattern does not match if found.
    // fn find_negation(self, input: &I) -> Option<usize>;
}

unsafe impl<'i, F> Pattern<Bytes<'i>> for F
where
    F: FnMut(u8) -> bool,
{
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .position(self)
            .map(|i| (i, 1))
    }
}

unsafe impl<'i, F> Pattern<String<'i>> for F
where
    F: FnMut(char) -> bool,
{
    fn find(mut self, input: &String<'i>) -> Option<(usize, usize)> {
        for (i, c) in input.as_dangerous().char_indices() {
            if !(self)(c) {
                return Some((i, c.len_utf8()));
            }
        }
        None
    }
}

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    #[cfg(feature = "memchr")]
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        memchr::memchr(self, input.as_dangerous()).map(|index| (index, 1))
    }
    #[cfg(not(feature = "memchr"))]
    fn find(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .position(|b| b == self)
            .map(|index| (index, 1))
    }
}

unsafe impl<'i> Pattern<String<'i>> for char {
    // FIXME: add SIMD optimised search
    fn find(self, input: &String<'i>) -> Option<(usize, usize)> {
        input.as_dangerous().char_indices().find_map(|(i, c)| {
            if c == self {
                Some((i, c.len_utf8()))
            } else {
                None
            }
        })
    }
}
