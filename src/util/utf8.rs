use core::str;

// Source: <rust-source>/core/str/mod.rs
// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_LENGTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
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

pub(crate) unsafe fn from_unchecked(bytes: &[u8]) -> &str {
    debug_assert!(str::from_utf8(bytes).is_ok());
    str::from_utf8_unchecked(bytes)
}

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub(crate) fn char_len(b: u8) -> usize {
    UTF8_CHAR_LENGTH[b as usize] as usize
}

pub(crate) struct CharIter<'i> {
    forward: usize,
    backward: usize,
    bytes: &'i [u8],
}

impl<'i> CharIter<'i> {
    pub(crate) fn new(bytes: &'i [u8]) -> Self {
        Self {
            bytes,
            forward: 0,
            backward: bytes.len(),
        }
    }

    pub(crate) fn as_slice(&self) -> &'i [u8] {
        &self.bytes[self.forward..self.backward]
    }

    pub(crate) fn forward(&self) -> &'i str {
        // SAFETY: bytes before this forward increasing index is valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.forward]) }
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
        if self.forward == self.backward {
            None
        } else {
            let result = match first_codepoint(self.head()) {
                Ok(c) => {
                    let forward = self.forward.saturating_add(c.len_utf8());
                    // If the parsing of the character goes over the reader's
                    // backward bound raise an error.
                    if forward > self.backward {
                        Err(InvalidChar(None))
                    } else {
                        self.forward = forward;
                        Ok(c)
                    }
                }
                Err(error_len) => Err(InvalidChar(error_len)),
            };
            Some(result)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.backward - self.forward;
        (remaining, Some(remaining))
    }
}

impl<'i> DoubleEndedIterator for CharIter<'i> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.forward == self.backward {
            None
        } else {
            let result = match last_codepoint(self.tail()) {
                Ok(c) => {
                    let backward = self.backward.saturating_sub(c.len_utf8());
                    // If the parsing of the character goes over the reader's
                    // forward bound raise an error.
                    if backward < self.forward {
                        Err(InvalidChar(None))
                    } else {
                        self.backward = backward;
                        Ok(c)
                    }
                }
                Err(error_len) => Err(InvalidChar(error_len)),
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

#[cfg_attr(test, derive(Debug))]
pub(crate) struct InvalidChar(Option<usize>);

impl InvalidChar {
    pub(crate) fn error_len(&self) -> Option<usize> {
        self.0
    }
}

#[inline(always)]
fn first_codepoint(bytes: &[u8]) -> Result<char, Option<usize>> {
    if let Some(first_byte) = bytes.first() {
        let len = char_len(*first_byte);
        if bytes.len() >= len {
            return parse_char(&bytes[..len]);
        }
    }
    Err(None)
}

#[inline(always)]
fn last_codepoint(bytes: &[u8]) -> Result<char, Option<usize>> {
    if bytes.is_empty() {
        return Err(None);
    }
    for (i, byte) in (1..=4).zip(bytes.iter().rev().copied()) {
        if !is_cont_byte(byte) && char_len(byte) == i {
            let last_index = bytes.len() - i;
            return parse_char(&bytes[last_index..]);
        }
    }
    Err(None)
}

#[inline(always)]
fn parse_char(bytes: &[u8]) -> Result<char, Option<usize>> {
    match str::from_utf8(bytes) {
        Ok(s) => match s.chars().next() {
            Some(c) => Ok(c),
            None => Err(None),
        },
        Err(e) => Err(e.error_len()),
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
