use crate::util::utf8::CharIter;

use super::unit::{byte_display_width, char_display_width};

#[derive(Copy, Clone)]
pub(super) struct Unit {
    pub(super) len_utf8: usize,
    pub(super) display_cost: usize,
}

impl Unit {
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

type UnitIterFn = fn(&mut &[u8], bool) -> Option<Result<Unit, ()>>;

#[derive(Clone)]
pub(super) struct UnitIter<'a> {
    bytes: &'a [u8],
    modifier: bool,
    next_front: UnitIterFn,
    next_back: UnitIterFn,
}

impl<'a> UnitIter<'a> {
    pub(super) fn new_byte(bytes: &'a [u8], show_ascii: bool) -> Self {
        Self {
            bytes,
            modifier: show_ascii,
            next_front: byte_next_front,
            next_back: byte_next_back,
        }
    }

    pub(super) fn new_char(bytes: &'a [u8], cjk: bool) -> Self {
        Self {
            bytes,
            modifier: cjk,
            next_front: char_next_front,
            next_back: char_next_back,
        }
    }

    pub(super) fn has_next(&self) -> bool {
        !self.bytes.is_empty()
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        self.bytes
    }

    pub(super) fn next_front(&mut self) -> Option<Result<Unit, ()>> {
        (self.next_front)(&mut self.bytes, self.modifier)
    }

    pub(super) fn next_back(&mut self) -> Option<Result<Unit, ()>> {
        (self.next_back)(&mut self.bytes, self.modifier)
    }

    pub(super) fn rev(self) -> Self {
        Self {
            bytes: self.bytes,
            modifier: self.modifier,
            next_front: self.next_back,
            next_back: self.next_front,
        }
    }

    pub(super) fn skip_head_bytes(mut self, len: usize) -> Self {
        self.bytes = if self.bytes.len() > len {
            &self.bytes[len..]
        } else {
            &[]
        };
        self
    }

    pub(super) fn skip_tail_bytes(mut self, len: usize) -> Self {
        self.bytes = if self.bytes.len() > len {
            &self.bytes[..self.bytes.len() - len]
        } else {
            &[]
        };
        self
    }
}

///////////////////////////////////////////////////////////////////////////////

fn byte_next_front(bytes: &mut &[u8], show_ascii: bool) -> Option<Result<Unit, ()>> {
    if bytes.is_empty() {
        None
    } else {
        let unit = Unit::byte(bytes[0], show_ascii);
        *bytes = &bytes[1..];
        Some(Ok(unit))
    }
}

fn byte_next_back(bytes: &mut &[u8], show_ascii: bool) -> Option<Result<Unit, ()>> {
    if bytes.is_empty() {
        None
    } else {
        let end = bytes.len() - 1;
        let unit = Unit::byte(bytes[end], show_ascii);
        *bytes = &bytes[..end];
        Some(Ok(unit))
    }
}

///////////////////////////////////////////////////////////////////////////////

fn char_next_front(bytes: &mut &[u8], cjk: bool) -> Option<Result<Unit, ()>> {
    let mut iter = CharIter::new(bytes);
    let result = iter
        .next()
        .map(|result| result.map(|c| Unit::unicode(c, cjk)).map_err(drop));
    *bytes = iter.as_slice();
    result
}

fn char_next_back(bytes: &mut &[u8], cjk: bool) -> Option<Result<Unit, ()>> {
    let mut iter = CharIter::new(bytes);
    let result = iter
        .next_back()
        .map(|result| result.map(|c| Unit::unicode(c, cjk)).map_err(drop));
    *bytes = iter.as_slice();
    result
}
