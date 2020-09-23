use core::str;

use crate::input::{input, Input};

// Source: <rust-source>/core/str/mod.rs
// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
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
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

/// Checks whether the byte is a UTF-8 continuation byte (i.e., starts with the
/// bits `10`).
#[inline]
fn utf8_is_cont_byte(byte: u8) -> bool {
    (byte & !CONT_MASK) == TAG_CONT_U8
}

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub(crate) fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

pub(crate) struct CharIter<'i> {
    forward: usize,
    backward: usize,
    bytes: &'i [u8],
}

impl<'i> CharIter<'i> {
    pub(crate) fn new(input: &'i Input) -> Self {
        Self {
            bytes: input.as_dangerous(),
            forward: 0,
            backward: input.len(),
        }
    }

    pub(crate) fn head(&self) -> &'i Input {
        input(&self.bytes[..self.forward])
    }

    pub(crate) fn tail(&self) -> &'i Input {
        input(&self.bytes[self.forward..])
    }
}

impl<'i> Iterator for CharIter<'i> {
    type Item = Result<char, InvalidChar>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let tail = self.tail();
        if tail.is_empty() {
            None
        } else {
            let result = first_codepoint(tail.as_dangerous()).map(|c| {
                self.index += c.len_utf8();
                c
            });
            Some(result)
        }
    }
}

impl<'i> DoubleEndedIterator for CharIter<'i> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let head = self.head();
        if head.is_empty() {
            None
        } else {
            let result = last_codepoint(head.as_dangerous()).map(|c| {
                self.index -= c.len_utf8();
                c
            });
            Some(result)
        }
    }
}

pub(crate) struct InvalidChar(());

#[inline(always)]
fn first_codepoint(bytes: &[u8]) -> Result<char, InvalidChar> {
    if let Some(first_byte) = bytes.first() {
        let len = utf8_char_width(*first_byte);
        if bytes.len() >= len {
            return parse_char(&bytes[..len]);
        }
    }
    Err(InvalidChar(()))
}

#[inline(always)]
fn last_codepoint(bytes: &[u8]) -> Result<char, InvalidChar> {
    let mut i = bytes.len();
    while i > bytes.len().saturating_sub(4) {
        let byte = bytes[i];
        if !utf8_is_cont_byte(byte) {
            if utf8_char_width(byte) == bytes.len() - i {
                return parse_char(&bytes[i..]);
            }
        }
    }
    Err(InvalidChar(()))
}

fn parse_char(bytes: &[u8]) -> Result<char, InvalidChar> {
    if let Ok(s) = str::from_utf8(bytes) {
        Ok(s.chars().next().unwrap())
    } else {
        Err(InvalidChar(()))
    }
}
