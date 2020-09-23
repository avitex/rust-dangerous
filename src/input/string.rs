use core::marker::PhantomData;

use crate::error::{ExpectedLength, ExpectedValid};
use crate::input::Input;

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

// /// Checks whether the byte is a UTF-8 continuation byte (i.e., starts with the
// /// bits `10`).
// #[inline]
// fn utf8_is_cont_byte(byte: u8) -> bool {
//     (byte & !CONT_MASK) == TAG_CONT_U8
// }

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub(crate) fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

pub(crate) struct CharIter<'i, E> {
    input: &'i Input,
    marker: PhantomData<E>,
}

impl<'i, E> CharIter<'i, E> {
    pub(crate) fn new(input: &'i Input) -> Self {
        Self {
            input,
            marker: PhantomData,
        }
    }

    pub(crate) fn as_input(&self) -> &Input {
        self.input
    }
}

impl<'i, E> Iterator for CharIter<'i, E>
where
    E: From<ExpectedValid<'i>>,
    E: From<ExpectedLength<'i>>,
{
    type Item = Result<char, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            None
        } else {
            let result = self.input.split_char("next char").map(|(c, remaining)| {
                self.input = remaining;
                c
            });
            Some(result)
        }
    }
}
