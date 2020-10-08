#[cfg(feature = "unicode")]
use unicode_width::UnicodeWidthChar;

use core::fmt::{self, Write};

pub(super) struct ByteDisplay {
    b: u8,
    show_ascii: bool,
}

impl ByteDisplay {
    pub(super) fn new(b: u8, show_ascii: bool) -> Self {
        Self { b, show_ascii }
    }

    pub(crate) fn width(&self) -> usize {
        if self.show_ascii {
            match self.b {
                b'\"' | b'\'' | b'\n' | b'\r' | b'\t' => "'\\x'".len(),
                c if c.is_ascii_graphic() => "'x'".len(),
                _ => "xx".len(),
            }
        } else {
            "xx".len()
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut W) -> fmt::Result {
        if self.show_ascii {
            match self.b {
                b'\"' => w.write_str("'\\\"'"),
                b'\'' => w.write_str("'\\''"),
                b'\n' => w.write_str("'\\n'"),
                b'\r' => w.write_str("'\\r'"),
                b'\t' => w.write_str("'\\t'"),
                b if b.is_ascii_graphic() => {
                    w.write_char('\'')?;
                    w.write_char(b as char)?;
                    w.write_char('\'')
                }
                b => write!(w, "{:0>2x}", b),
            }
        } else {
            write!(w, "{:0>2x}", self.b)
        }
    }
}

pub(super) struct CharDisplay {
    c: char,
    cjk: bool,
}

impl CharDisplay {
    pub(super) fn new(c: char, cjk: bool) -> Self {
        Self { c, cjk }
    }

    pub(crate) fn width(&self) -> usize {
        self.c
            .escape_debug()
            .fold(0, |acc, c| acc + unicode_width(c, self.cjk))
    }

    pub(crate) fn write<W: Write>(&self, w: &mut W) -> fmt::Result {
        for c in self.c.escape_debug() {
            w.write_char(c)?;
        }
        Ok(())
    }
}

#[cfg(feature = "unicode")]
#[inline]
fn unicode_width(c: char, cjk: bool) -> usize {
    if cjk { c.width_cjk() } else { c.width() }.unwrap_or(1)
}

#[cfg(not(feature = "unicode"))]
#[inline]
fn unicode_width(c: char, _cjk: bool) -> usize {
    1
}
