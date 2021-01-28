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

pub(super) trait SectionUnitIter<E>:
    DoubleEndedIterator<Item = Result<SectionUnit, E>> + Clone
{
    fn as_slice(&self) -> &[u8];

    fn skip_head_bytes(self, len: usize) -> Self;

    fn skip_tail_bytes(self, len: usize) -> Self;
}

///////////////////////////////////////////////////////////////////////////////

pub(super) struct ByteSectionUnitIter<'a> {
    iter: slice::Iter<'a, u8>,
    show_ascii: bool,
}

impl<'a> ByteSectionUnitIter<'a> {
    pub(super) fn new(bytes: &'a [u8], show_ascii: bool) -> Self {
        Self {
            iter: bytes.iter(),
            show_ascii,
        }
    }
}

impl<'a> Iterator for ByteSectionUnitIter<'a> {
    type Item = Result<SectionUnit, Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|b| Ok(SectionUnit::byte(*b, self.show_ascii)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for ByteSectionUnitIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|b| Ok(SectionUnit::byte(*b, self.show_ascii)))
    }
}

impl<'a> SectionUnitIter<Infallible> for ByteSectionUnitIter<'a> {
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

    fn as_slice(&self) -> &[u8] {
        self.iter.as_slice()
    }
}

impl<'a> Clone for ByteSectionUnitIter<'a> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            show_ascii: self.show_ascii,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(super) struct CharSectionUnitIter<'a> {
    iter: CharIter<'a>,
    cjk: bool,
}

impl<'a> CharSectionUnitIter<'a> {
    pub(super) fn new(bytes: &'a [u8], cjk: bool) -> Self {
        Self {
            iter: CharIter::new(bytes),
            cjk,
        }
    }
}

impl<'a> Iterator for CharSectionUnitIter<'a> {
    type Item = Result<SectionUnit, InvalidChar>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|r| r.map(|c| SectionUnit::unicode(c, self.cjk)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for CharSectionUnitIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|r| r.map(|c| SectionUnit::unicode(c, self.cjk)))
    }
}

impl<'a> SectionUnitIter<InvalidChar> for CharSectionUnitIter<'a> {
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

    fn as_slice(&self) -> &[u8] {
        self.iter.as_slice()
    }
}

impl<'a> Clone for CharSectionUnitIter<'a> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            cjk: self.cjk,
        }
    }
}
