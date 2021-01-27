// # Safety note about searching for substrings for `Pattern`.
//
// As we are looking for exact slice matches, the needle and haystack are both
// UTF-8, we are searching forward and the fact the first byte of a UTF-8
// character cannot clash with a continuation byte we can safely search as just
// raw bytes and return those indexes.
//
// For searching within a string we care that we don't return an index that
// isn't a char boundary, but with the above conditions this is impossible.
//
// Note in the below UTF-8 byte sequence table continuation bytes (bytes 2-4)
// cannot clash with the first byte. This means when the first byte of the
// needle string matches the first byte of the haystack string, we are at a
// valid char boundary and if rest of the needle matches the indexes we return
// are valid.
//
// | Length | Byte 1   | Byte 2   | Byte 3   | Byte 4   |
// | ------ | -------- | -------- | -------- | -------- |
// | 1      | 0xxxxxxx |          |          |          |
// | 2      | 110xxxxx | 10xxxxxx |          |          |
// | 3      | 1110xxxx | 10xxxxxx | 10xxxxxx |          |
// | 4      | 11110xxx | 10xxxxxx | 10xxxxxx | 10xxxxxx |

use crate::input::{Bytes, String};
use crate::util::fast;

/// Implemented for structures that can be found within an
/// [`Input`](crate::Input).
///
/// You can search for a `char` or `&str` within either `Bytes` or `String`, but
/// only a `u8` and `&[u8]` within `Bytes`.
///
/// Empty slices are invalid patterns and have the following behaviour:
///
/// - Finding a match of a empty slice pattern will return `None`.
/// - Finding a reject of a empty slice pattern will return `Some(0)`.
///
/// With the `simd` feature enabled pattern searches are SIMD optimised where
/// possible.
///
/// With the `regex` feature enabled, you can search for regex patterns.
///
/// # Safety
///
/// The implementation must return valid indexes and lengths for splitting input
/// as these are not checked.
pub unsafe trait Pattern<I> {
    /// Returns the byte index and byte length of the first match and `None` if
    /// there was no match.
    fn find_match(self, input: &I) -> Option<(usize, usize)>;

    /// Returns the byte index of the first reject and `None` if there was no
    /// reject.
    fn find_reject(self, input: &I) -> Option<usize>;
}

///////////////////////////////////////////////////////////////////////////////
// Fn pattern

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

///////////////////////////////////////////////////////////////////////////////
// Token pattern

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_u8_match(self, input.as_dangerous()).map(|index| (index, 1))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_u8_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i> Pattern<Bytes<'i>> for char {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_char_match(self, input.as_dangerous()).map(|index| (index, self.len_utf8()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_char_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i> Pattern<String<'i>> for char {
    fn find_match(self, input: &String<'i>) -> Option<(usize, usize)> {
        fast::find_char_match(self, input.as_dangerous().as_bytes())
            .map(|index| (index, self.len_utf8()))
    }

    fn find_reject(self, input: &String<'i>) -> Option<usize> {
        fast::find_char_reject(self, input.as_dangerous().as_bytes())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Sub-slice pattern

unsafe impl<'i> Pattern<Bytes<'i>> for &[u8] {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self, input.as_dangerous()).map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i, const N: usize> Pattern<Bytes<'i>> for &[u8; N] {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self, input.as_dangerous()).map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i> Pattern<Bytes<'i>> for &str {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self.as_bytes(), input.as_dangerous())
            .map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self.as_bytes(), input.as_dangerous())
    }
}

unsafe impl<'i> Pattern<String<'i>> for &str {
    #[inline]
    fn find_match(self, input: &String<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self.as_bytes(), input.as_dangerous().as_bytes())
            .map(|index| (index, self.len()))
    }

    #[inline]
    fn find_reject(self, input: &String<'i>) -> Option<usize> {
        fast::find_slice_reject(self.as_bytes(), input.as_dangerous().as_bytes())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Regex pattern

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
