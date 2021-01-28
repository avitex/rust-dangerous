use core::convert::Infallible;
use core::slice;

use crate::util::utf8::{CharIter, InvalidChar};

use super::unit::{byte_display_width, char_display_width};

#[derive(Copy, Clone)]
pub(super) struct SectionUnit {
    pub(super) len_utf8: usize,
    pub(super) display_cost: usize,
}

impl SectionUnit {
    pub(super) fn byte(b: u8, show_ascii: bool) -> Self {
        Self {
            display_cost: byte_display_width(b, show_ascii),
            len_utf8: 1,
        }
    }

    pub(super) fn unicode(c: char, cjk: bool) -> Self {
        Self {
            display_cost: char_display_width(c, cjk),
            len_utf8: c.len_utf8(),
        }
    }
}

pub(super) trait UnitIterBase: Clone {
    type Error;

    fn has_next(&self) -> bool;

    fn next_front(&mut self) -> Option<Result<SectionUnit, Self::Error>>;

    fn next_back(&mut self) -> Option<Result<SectionUnit, Self::Error>>;
}

pub(super) trait UnitIter: UnitIterBase {
    fn as_slice(&self) -> &[u8];

    fn skip_head_bytes(self, len: usize) -> Self;

    fn skip_tail_bytes(self, len: usize) -> Self;

    fn rev(self) -> Rev<Self> {
        Rev(self)
    }
}

#[derive(Clone)]
pub struct Rev<T>(pub T);

impl<T> UnitIterBase for Rev<T>
where
    T: UnitIterBase,
{
    type Error = T::Error;

    fn has_next(&self) -> bool {
        self.0.has_next()
    }

    fn next_front(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.0.next_back()
    }

    fn next_back(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.0.next_front()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(super) struct ByteUnitIter<'a> {
    iter: slice::Iter<'a, u8>,
    show_ascii: bool,
}

impl<'a> ByteUnitIter<'a> {
    pub(super) fn new(bytes: &'a [u8], show_ascii: bool) -> Self {
        Self {
            iter: bytes.iter(),
            show_ascii,
        }
    }
}

impl<'a> UnitIterBase for ByteUnitIter<'a> {
    type Error = Infallible;

    fn has_next(&self) -> bool {
        !self.as_slice().is_empty()
    }

    fn next_front(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.iter
            .next()
            .map(|b| Ok(SectionUnit::byte(*b, self.show_ascii)))
    }

    fn next_back(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.iter
            .next_back()
            .map(|b| Ok(SectionUnit::byte(*b, self.show_ascii)))
    }
}

impl<'a> UnitIter for ByteUnitIter<'a> {
    fn as_slice(&self) -> &[u8] {
        self.iter.as_slice()
    }

    fn skip_head_bytes(mut self, len: usize) -> Self {
        if len > 0 {
            let _ = self.iter.nth(len - 1);
        }
        self
    }

    fn skip_tail_bytes(mut self, len: usize) -> Self {
        if len > 0 {
            let _ = self.iter.nth_back(len - 1);
        }
        self
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(super) struct CharUnitIter<'a> {
    iter: CharIter<'a>,
    cjk: bool,
}

impl<'a> CharUnitIter<'a> {
    pub(super) fn new(bytes: &'a [u8], cjk: bool) -> Self {
        Self {
            iter: CharIter::new(bytes),
            cjk,
        }
    }
}

impl<'a> UnitIterBase for CharUnitIter<'a> {
    type Error = InvalidChar;

    fn has_next(&self) -> bool {
        !self.as_slice().is_empty()
    }

    fn next_front(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.iter
            .next()
            .map(|r| r.map(|c| SectionUnit::unicode(c, self.cjk)))
    }

    fn next_back(&mut self) -> Option<Result<SectionUnit, Self::Error>> {
        self.iter
            .next_back()
            .map(|r| r.map(|c| SectionUnit::unicode(c, self.cjk)))
    }
}

impl<'a> UnitIter for CharUnitIter<'a> {
    fn as_slice(&self) -> &[u8] {
        self.iter.as_slice()
    }

    fn skip_head_bytes(mut self, len: usize) -> Self {
        let bytes = self.iter.as_slice();
        let bytes = if bytes.len() > len {
            &bytes[len..]
        } else {
            &[]
        };
        self.iter = CharIter::new(bytes);
        self
    }

    fn skip_tail_bytes(mut self, len: usize) -> Self {
        let bytes = self.iter.as_slice();
        let bytes = if bytes.len() > len {
            &bytes[..bytes.len() - len]
        } else {
            &[]
        };
        self.iter = CharIter::new(bytes);
        self
    }
}
