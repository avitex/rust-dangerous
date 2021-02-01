use crate::input::{Bytes, String};

/// A structure that can be found within an [`Input`](crate::Input).
///
/// # Safety
///
/// The implementation must returned valid indexes and lengths for splitting
/// input as these are not checked.
pub unsafe trait Pattern<I> {
    /// Returns the byte index and byte length of the first match and `None` if
    /// there was no match.
    fn find_match(self, input: &I) -> Option<(usize, usize)>;

    /// Returns the byte index of the first reject and `None` if there was no
    /// reject.
    fn find_reject(self, input: &I) -> Option<usize>;
}

unsafe impl<'i, F> Pattern<Bytes<'i>> for F
where
    F: FnMut(u8) -> bool,
{
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .position(self)
            .map(|i| (i, 1))
    }

    fn find_reject(mut self, input: &Bytes<'i>) -> Option<usize> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .enumerate()
            .find_map(|(i, b)| if (self)(b) { None } else { Some(i) })
    }
}

unsafe impl<'i, F> Pattern<String<'i>> for F
where
    F: FnMut(char) -> bool,
{
    fn find_match(mut self, input: &String<'i>) -> Option<(usize, usize)> {
        for (i, c) in input.as_dangerous().char_indices() {
            if !(self)(c) {
                return Some((i, c.len_utf8()));
            }
        }
        None
    }

    fn find_reject(mut self, input: &String<'i>) -> Option<usize> {
        input.as_dangerous().char_indices().find_map(
            |(i, b)| {
                if (self)(b) {
                    None
                } else {
                    Some(i)
                }
            },
        )
    }
}

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    #[cfg(feature = "memchr")]
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
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

    // FIXME: add SIMD optimised search
    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        Pattern::find_reject(|b| b == self, input)
    }
}

unsafe impl<'i> Pattern<String<'i>> for char {
    // FIXME: add SIMD optimised search
    fn find_match(self, input: &String<'i>) -> Option<(usize, usize)> {
        input.as_dangerous().char_indices().find_map(|(i, c)| {
            if c == self {
                Some((i, c.len_utf8()))
            } else {
                None
            }
        })
    }

    // FIXME: add SIMD optimised search
    fn find_reject(self, input: &String<'i>) -> Option<usize> {
        Pattern::find_reject(|c| c == self, input)
    }
}

#[cfg(feature = "regex")]
unsafe impl<'i> Pattern<String<'i>> for &regex::Regex {
    fn find_match(self, input: &String<'i>) -> Option<(usize, usize)> {
        regex::Regex::find(self, input.as_dangerous()).map(|m| (m.start(), m.end() - m.start()))
    }

    fn find_reject(self, input: &String<'i>) -> Option<usize> {
        let mut maybe_reject = 0;
        loop {
            match regex::Regex::find_at(self, input.as_dangerous(), maybe_reject) {
                Some(m) if input.as_dangerous().len() == m.end() => return None,
                Some(m) => {
                    maybe_reject = m.end();
                }
                None => return Some(maybe_reject),
            }
        }
    }
}

#[cfg(feature = "regex")]
unsafe impl<'i> Pattern<Bytes<'i>> for &regex::bytes::Regex {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        regex::bytes::Regex::find(self, input.as_dangerous())
            .map(|m| (m.start(), m.end() - m.start()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        let mut maybe_reject = 0;
        loop {
            match regex::bytes::Regex::find_at(self, input.as_dangerous(), maybe_reject) {
                Some(m) if input.len() == m.end() => return None,
                Some(m) => {
                    maybe_reject = m.end();
                }
                None => return Some(maybe_reject),
            }
        }
    }
}
