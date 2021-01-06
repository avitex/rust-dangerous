#[cfg(feature = "unicode")]
use unicode_width::UnicodeWidthChar;

use crate::fmt::{self, Write};

pub(super) fn byte_display_width(b: u8, show_ascii: bool) -> usize {
    if show_ascii {
        match b {
            b'\"' | b'\'' | b'\n' | b'\r' | b'\t' => "'\\x'".len(),
            c if c.is_ascii_graphic() => "'x'".len(),
            _ => "xx".len(),
        }
    } else {
        "xx".len()
    }
}

pub(super) fn byte_display_write<W: Write + ?Sized>(
    b: u8,
    show_ascii: bool,
    w: &mut W,
) -> fmt::Result {
    if show_ascii {
        match b {
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
            b => w.write_hex(b),
        }
    } else {
        w.write_hex(b)
    }
}

pub(super) fn char_display_width(c: char, cjk: bool) -> usize {
    c.escape_debug()
        .fold(0, |acc, c| acc + unicode_width(c, cjk))
}

pub(super) fn char_display_write<W: Write + ?Sized>(c: char, w: &mut W) -> fmt::Result {
    for c in c.escape_debug() {
        w.write_char(c)?;
    }
    Ok(())
}

#[cfg(feature = "unicode")]
#[inline]
fn unicode_width(c: char, cjk: bool) -> usize {
    if cjk { c.width_cjk() } else { c.width() }.unwrap_or(1)
}

#[cfg(not(feature = "unicode"))]
#[inline]
fn unicode_width(_c: char, _cjk: bool) -> usize {
    1
}
