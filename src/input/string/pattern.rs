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

use crate::input::{Pattern, String};
use crate::util::fast;

///////////////////////////////////////////////////////////////////////////////
// Fn pattern

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
