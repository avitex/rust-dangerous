use core::slice;

use crate::string::{utf8_char_display_width, CharIter};

#[derive(Clone, Copy)]
pub(crate) struct Element {
    pub(crate) byte_len: usize,
    pub(crate) display_cost: usize,
}

impl Element {
    fn byte(byte: u8, show_ascii: bool) -> Self {
        let display_cost = if show_ascii && byte.is_ascii_graphic() {
            b"'x'".len()
        } else {
            b"ff".len()
        };
        Self {
            display_cost,
            byte_len: 1,
        }
    }

    fn utf8(c: char, cjk: bool) -> Self {
        Self {
            display_cost: utf8_char_display_width(c, cjk),
            byte_len: c.len_utf8(),
        }
    }
}

pub(crate) trait ElementIter:
    Clone + DoubleEndedIterator<Item = Result<Element, ()>>
{
    fn as_slice(&self) -> &[u8];

    fn skip_head_bytes(self, len: usize) -> Self;

    fn skip_tail_bytes(self, len: usize) -> Self;
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct ByteElementIter<'a> {
    iter: slice::Iter<'a, u8>,
    show_ascii: bool,
}

impl<'a> ByteElementIter<'a> {
    pub(crate) fn new(bytes: &'a [u8], show_ascii: bool) -> Self {
        Self {
            iter: bytes.iter(),
            show_ascii,
        }
    }
}

impl<'a> Iterator for ByteElementIter<'a> {
    type Item = Result<Element, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|b| Ok(Element::byte(*b, self.show_ascii)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for ByteElementIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|b| Ok(Element::byte(*b, self.show_ascii)))
    }
}

impl<'a> ElementIter for ByteElementIter<'a> {
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

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct CharElementIter<'a> {
    iter: CharIter<'a>,
    cjk: bool,
}

impl<'a> CharElementIter<'a> {
    pub(crate) fn new(bytes: &'a [u8], cjk: bool) -> Self {
        Self {
            iter: CharIter::new(bytes),
            cjk,
        }
    }
}

impl<'a> Iterator for CharElementIter<'a> {
    type Item = Result<Element, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|r| r.map(|c| Element::utf8(c, self.cjk)).map_err(drop))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator for CharElementIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|r| r.map(|c| Element::utf8(c, self.cjk)).map_err(drop))
    }
}

impl<'a> ElementIter for CharElementIter<'a> {
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
            &bytes[..len]
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

///////////////////////////////////////////////////////////////////////////////

pub(crate) enum Alternate<T> {
    Back(T),
    Front(T),
}

pub(crate) struct AlternatingIter<I> {
    inner: I,
    front: bool,
}

impl<I> AlternatingIter<I> {
    pub(crate) fn front(iter: I) -> Self {
        Self {
            inner: iter,
            front: true,
        }
    }
}

impl<I> Iterator for AlternatingIter<I>
where
    I: DoubleEndedIterator,
{
    type Item = Alternate<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front {
            self.front = false;
            self.inner.next().map(Alternate::Front)
        } else {
            self.front = true;
            self.inner.next_back().map(Alternate::Back)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> ExactSizeIterator for AlternatingIter<I> where I: ExactSizeIterator + DoubleEndedIterator {}
