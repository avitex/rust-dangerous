//! Display support.

mod error;
mod input;
mod section;
mod unit;

use core::fmt::{Formatter, Result};

pub use self::error::ErrorDisplay;
pub use self::input::{InputDisplay, PreferredFormat};

/// Library specific display trait that accepts a [`Write`] without requiring a
/// formatter.
pub trait DisplayBase {
    /// Formats `self` given the provided [`Write`].
    ///
    /// # Errors
    ///
    /// Returns a [`core::fmt::Error`] if failed to write.
    fn fmt(&self, w: &mut dyn Write) -> Result;
}

impl<T> DisplayBase for &T
where
    T: DisplayBase,
{
    fn fmt(&self, w: &mut dyn Write) -> Result {
        (**self).fmt(w)
    }
}

impl DisplayBase for &'static str {
    fn fmt(&self, w: &mut dyn Write) -> Result {
        w.write_str(self)
    }
}

/// Library specific [`Write`] trait for formatting.
pub trait Write {
    /// Writes a string slice into this writer, returning whether the write
    /// succeeded.
    ///
    /// # Errors
    ///
    /// Returns a [`core::fmt::Error`] if failed to write.
    fn write_str(&mut self, s: &str) -> Result;

    /// Writes a char into this writer, returning whether the write succeeded.
    ///
    /// # Errors
    ///
    /// Returns a [`core::fmt::Error`] if failed to write.
    fn write_char(&mut self, c: char) -> Result;

    /// Writes a usize into this writer, returning whether the write succeeded.
    ///
    /// # Errors
    ///
    /// Returns a [`core::fmt::Error`] if failed to write.
    fn write_usize(&mut self, v: usize) -> Result;

    /// Writes a byte as hex into this writer, returning whether the write
    /// succeeded.
    ///
    /// The byte as hex must be always two characters long (zero-padded).
    ///
    /// # Errors
    ///
    /// Returns a [`core::fmt::Error`] if failed to write.
    fn write_hex(&mut self, b: u8) -> Result {
        fn digit(b: u8) -> char {
            if b > 9 {
                (b'a' + (b - 10)) as char
            } else {
                (b'0' + b) as char
            }
        }
        self.write_char(digit(b >> 4))?;
        self.write_char(digit(b & 0x0F))
    }
}

impl<T> Write for &mut T
where
    T: Write,
{
    fn write_str(&mut self, s: &str) -> Result {
        (**self).write_str(s)
    }

    fn write_char(&mut self, c: char) -> Result {
        (**self).write_char(c)
    }

    fn write_usize(&mut self, v: usize) -> Result {
        (**self).write_usize(v)
    }

    fn write_hex(&mut self, b: u8) -> Result {
        (**self).write_hex(b)
    }
}

impl<'a> Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> Result {
        core::fmt::Write::write_str(self, s)
    }

    fn write_char(&mut self, c: char) -> Result {
        core::fmt::Write::write_char(self, c)
    }

    fn write_usize(&mut self, v: usize) -> Result {
        core::fmt::Display::fmt(&v, self)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn byte_count(w: &mut dyn Write, count: usize) -> Result {
    match count {
        0 => w.write_str("no bytes"),
        1 => w.write_str("1 byte"),
        n => {
            w.write_usize(n)?;
            w.write_str(" bytes")
        }
    }
}
