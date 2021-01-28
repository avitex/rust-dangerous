use core::str;

// Source: https://github.com/rust-lang/rust/blob/master/library/core/src/str/validations.rs
// https://tools.ietf.org/html/rfc3629
#[rustfmt::skip]
static UTF8_CHAR_LENGTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x0F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x2F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x4F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x6F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xAF
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xCF
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !`CONT_MASK`) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

/// Checks whether the byte is a UTF-8 continuation byte (i.e., starts with the
/// bits `10`).
#[inline]
fn is_cont_byte(byte: u8) -> bool {
    (byte & !CONT_MASK) == TAG_CONT_U8
}

/// Returns a str slice from a byte slice without validation.
#[inline]
pub(crate) unsafe fn from_unchecked(bytes: &[u8]) -> &str {
    debug_assert!(str::from_utf8(bytes).is_ok());
    str::from_utf8_unchecked(bytes)
}

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub(crate) fn char_len(b: u8) -> usize {
    UTF8_CHAR_LENGTH[b as usize] as usize
}

/// Returns the first UTF-8 codepoint if valid within bytes.
#[inline(always)]
fn first_codepoint(bytes: &[u8]) -> Result<char, InvalidChar> {
    if let Some(first_byte) = bytes.first() {
        let len = char_len(*first_byte);
        if bytes.len() >= len {
            return parse_char(&bytes[..len]);
        }
    }
    Err(InvalidChar { error_len: None })
}

/// Returns the last UTF-8 codepoint if valid within bytes.
#[inline(always)]
fn last_codepoint(bytes: &[u8]) -> Result<char, InvalidChar> {
    if bytes.is_empty() {
        return Err(InvalidChar { error_len: None });
    }
    for (i, byte) in (1..=4).zip(bytes.iter().rev().copied()) {
        if !is_cont_byte(byte) && char_len(byte) == i {
            let last_index = bytes.len() - i;
            return parse_char(&bytes[last_index..]);
        }
    }
    Err(InvalidChar { error_len: None })
}

/// Parses a single char from bytes.
#[inline(always)]
fn parse_char(bytes: &[u8]) -> Result<char, InvalidChar> {
    match str::from_utf8(bytes) {
        Ok(s) => match s.chars().next() {
            Some(c) => Ok(c),
            None => Err(InvalidChar { error_len: None }),
        },
        Err(e) => Err(InvalidChar {
            error_len: e.error_len(),
        }),
    }
}

///////////////////////////////////////////////////////////////////////////////
// InvalidChar

#[cfg_attr(test, derive(Debug))]
pub(crate) struct InvalidChar {
    error_len: Option<usize>,
}

impl InvalidChar {
    pub(crate) fn error_len(&self) -> Option<usize> {
        self.error_len
    }
}

///////////////////////////////////////////////////////////////////////////////
// CharIter

/// Char iterator over unvalidated bytes.
pub(crate) struct CharIter<'i> {
    forward: usize,
    backward: usize,
    bytes: &'i [u8],
}

impl<'i> CharIter<'i> {
    /// Creates a new char iterator from a byte slice.
    pub(crate) fn new(bytes: &'i [u8]) -> Self {
        Self {
            bytes,
            forward: 0,
            backward: bytes.len(),
        }
    }

    /// Returns the remaining slice.
    pub(crate) fn as_slice(&self) -> &'i [u8] {
        &self.bytes[self.forward..self.backward]
    }

    /// Returns the `str` consumed from the front.
    pub(crate) fn as_forward(&self) -> &'i str {
        // SAFETY: bytes before this forward increasing index is valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.forward]) }
    }

    fn is_done(&self) -> bool {
        self.forward == self.backward
    }

    fn head(&self) -> &'i [u8] {
        &self.bytes[self.forward..]
    }

    fn tail(&self) -> &'i [u8] {
        &self.bytes[..self.backward]
    }
}

impl<'i> Iterator for CharIter<'i> {
    type Item = Result<char, InvalidChar>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            None
        } else {
            let result = match first_codepoint(self.head()) {
                Ok(c) => {
                    let forward = self.forward.saturating_add(c.len_utf8());
                    // If the parsing of the character goes over the reader's
                    // backward bound raise an error.
                    if forward > self.backward {
                        Err(InvalidChar { error_len: None })
                    } else {
                        self.forward = forward;
                        Ok(c)
                    }
                }
                Err(err) => Err(err),
            };
            Some(result)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.is_done() {
            (0, None)
        } else {
            (1, None)
        }
    }
}

impl<'i> DoubleEndedIterator for CharIter<'i> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            None
        } else {
            let result = match last_codepoint(self.tail()) {
                Ok(c) => {
                    let backward = self.backward.saturating_sub(c.len_utf8());
                    // If the parsing of the character goes over the reader's
                    // forward bound raise an error.
                    if backward < self.forward {
                        Err(InvalidChar { error_len: None })
                    } else {
                        self.backward = backward;
                        Ok(c)
                    }
                }
                Err(err) => Err(err),
            };
            Some(result)
        }
    }
}

impl<'i> Clone for CharIter<'i> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_iter() {
        let mut char_iter = CharIter::new("\u{10348}a\u{10347}".as_bytes());
        assert_eq!(char_iter.next().unwrap().unwrap(), '\u{10348}');
        assert_eq!(char_iter.next_back().unwrap().unwrap(), '\u{10347}');
        assert_eq!(char_iter.next().unwrap().unwrap(), 'a');
    }

    #[test]
    fn test_last_codepoint() {
        assert!(last_codepoint(b"").is_err());
        assert!(last_codepoint(b"\xFF").is_err());
        assert!(last_codepoint(b"a\xFF").is_err());
        assert_eq!(last_codepoint(b"a").unwrap(), 'a');
        assert_eq!(last_codepoint(b"ab").unwrap(), 'b');
        assert_eq!(
            last_codepoint("a\u{10348}".as_bytes()).unwrap(),
            '\u{10348}'
        );
        assert_eq!(last_codepoint("\u{10348}".as_bytes()).unwrap(), '\u{10348}');
    }

    #[test]
    fn test_first_codepoint() {
        assert!(first_codepoint(b"").is_err());
        assert!(first_codepoint(b"\xFF").is_err());
        assert!(first_codepoint(b"\xFFa").is_err());
        assert_eq!(first_codepoint(b"a").unwrap(), 'a');
        assert_eq!(first_codepoint(b"ab").unwrap(), 'a');
        assert_eq!(
            first_codepoint("\u{10348}a".as_bytes()).unwrap(),
            '\u{10348}'
        );
        assert_eq!(
            first_codepoint("\u{10348}".as_bytes()).unwrap(),
            '\u{10348}'
        );
    }
}
