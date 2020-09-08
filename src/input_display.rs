use core::fmt::{self, Write};
use core::result::Result;

use crate::error::Invalid;
use crate::input::Input;

const DEFAULT_SECTION: Section = Section::HeadTail { max: 1024 };

/// Provides configurable [`Input`] formatting.
///
/// Defaults to formatting an [`Input`] to max `1024` bytes or UTF-8 code points.
///
/// # Format string options
///  
/// | Option     | `"heya ♥"`                  | `&[0xFF, 0xFF, b'a']` |
/// | ---------- | --------------------------- | --------------------- |
/// | `"{}"`     | `[68 65 79 61 20 e2 99 a5]` | `[ff ff 61]`          |
/// | `"{:#}"`   | `"heya ♥"`                  | `[ff ff 'a']`         |
/// | `"{:.2}"`  | `[68 .. a5]`                | `[ff .. 61]`          |
/// | `"{:#.2}"` | `"h".."♥"`                  | `[ff .. 'a']`         |
///
/// # Example
///
/// ```
/// let formatted = dangerous::input(b"hello")
///     .display()
///     .head_tail(3)
///     .to_string();
/// assert_eq!(formatted, "[68 65 .. 6f]");
/// ```
#[derive(Clone)]
pub struct InputDisplay<'i> {
    input: &'i Input,
    str_hint: bool,
    section: Option<Section>,
}

impl<'i> InputDisplay<'i> {
    /// Create a new `InputDisplay` given [`Input`].
    pub const fn new(input: &'i Input) -> Self {
        Self {
            input,
            str_hint: false,
            section: Some(DEFAULT_SECTION),
        }
    }

    /// Derive an `InputDisplay` from a [`fmt::Formatter`] with defaults.
    ///
    /// - Precision (eg. `{:.2}`) formatting sets the element limit.
    /// - Alternate/pretty (eg. `{:#}`) formatting enables the UTF-8 hint.
    pub fn from_formatter(input: &'i Input, f: &fmt::Formatter) -> Self {
        let format = Self::new(input).str_hint(f.alternate());
        match f.precision() {
            Some(max) => format.head_tail(max),
            None => format,
        }
    }

    /// Hint to the formatter that the [`Input`] is a UTF-8 `str`.
    pub fn str_hint(mut self, value: bool) -> Self {
        self.str_hint = value;
        self
    }

    /// Limit the [`Input`] to show `max` elements at the head of the input and
    /// at the tail.
    ///
    /// # Example
    ///
    /// ```
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().head_tail(4).to_string();
    ///
    /// assert_eq!(formatted, "[aa bb .. ee ff]");
    /// ```
    pub fn head_tail(mut self, max: usize) -> Self {
        self.section = Some(Section::HeadTail { max });
        self
    }

    /// Limit the [`Input`] to show `max` elements at the head of the input.
    ///
    /// # Example
    ///
    /// ```
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().head(4).to_string();
    ///
    /// assert_eq!(formatted, "[aa bb cc dd ..]");
    /// ```
    pub fn head(mut self, max: usize) -> Self {
        self.section = Some(Section::Head { max });
        self
    }

    /// Limit the [`Input`] to show `max` elements at the tail of the input.
    ///
    /// # Example
    ///
    /// ```
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().tail(4).to_string();
    ///
    /// assert_eq!(formatted, "[.. cc dd ee ff]");
    /// ```
    pub fn tail(mut self, max: usize) -> Self {
        self.section = Some(Section::Tail { max });
        self
    }

    /// Shows the all of the elements in the [`Input`].
    ///
    /// # Example
    ///
    /// ```
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().full().to_string();
    ///
    /// assert_eq!(formatted, "[aa bb cc dd ee ff]");
    /// ```
    pub fn full(mut self) -> Self {
        self.section = None;
        self
    }

    /// Writes the [`Input`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        if self.str_hint {
            if let Ok(s) = self.input.to_dangerous_str::<Invalid>() {
                write_str(w, s, self.section)
            } else {
                write_bytes(w, self.input, true, self.section)
            }
        } else {
            write_bytes(w, self.input, false, self.section)
        }
    }
}

impl<'i> fmt::Debug for InputDisplay<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

impl<'i> fmt::Display for InputDisplay<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
enum Section {
    Tail { max: usize },
    Head { max: usize },
    HeadTail { max: usize },
}

fn write_str<W>(w: &mut W, input: &str, section: Option<Section>) -> fmt::Result
where
    W: Write,
{
    match section {
        None => {
            w.write_char('"')?;
            w.write_str(input)?;
            w.write_char('"')
        }
        Some(Section::Head { max }) => {
            w.write_char('"')?;
            if write_str_contents_head(w, input, max)? {
                w.write_str("\"..")
            } else {
                w.write_char('"')
            }
        }
        Some(Section::Tail { max }) => {
            let count = input.chars().count();
            if count > max {
                w.write_str("..\"")?;
                write_str_contents_tail(w, input, count - max)?;
            } else {
                w.write_char('"')?;
                w.write_str(input)?;
            }
            w.write_char('"')
        }
        Some(Section::HeadTail { max }) => {
            w.write_char('"')?;
            let count = input.chars().count();
            if count > max {
                let (head_max, tail_max) = head_tail_max(max);
                if write_str_contents_head(w, input, head_max)? {
                    if tail_max == 0 {
                        w.write_str("\"..")
                    } else {
                        w.write_str("\"..\"")?;
                        write_str_contents_tail(w, input, count - tail_max)?;
                        w.write_char('"')
                    }
                } else {
                    w.write_char('"')
                }
            } else {
                w.write_str(input)?;
                w.write_char('"')
            }
        }
    }
}

fn write_bytes<W>(
    w: &mut W,
    input: &Input,
    show_ascii: bool,
    section: Option<Section>,
) -> fmt::Result
where
    W: Write,
{
    w.write_char('[')?;
    match section {
        None => {
            write_bytes_contents(w, input, show_ascii)?;
        }
        Some(Section::Head { max }) => {
            write_bytes_contents(w, input.split_max(max).0, show_ascii)?;
            if input.len() > max {
                w.write_str(" ..")?;
            }
        }
        Some(Section::Tail { max }) => {
            if input.len() > max {
                w.write_str(".. ")?;
            }
            write_bytes_contents(w, input.split_max(input.len() - max).1, show_ascii)?;
        }
        Some(Section::HeadTail { max }) => {
            if input.len() > max {
                let (head_max, tail_max) = head_tail_max(max);
                let head = input.split_max(head_max).0;
                let tail = input.split_max(input.len() - tail_max).1;
                write_bytes_contents(w, head, show_ascii)?;
                if tail_max == 0 {
                    w.write_str(" ..")?;
                } else {
                    w.write_str(" .. ")?;
                    write_bytes_contents(w, tail, show_ascii)?;
                }
            } else {
                write_bytes_contents(w, input, show_ascii)?;
            }
        }
    };
    w.write_char(']')
}

fn write_bytes_contents<W>(w: &mut W, input: &Input, show_ascii: bool) -> fmt::Result
where
    W: Write,
{
    let mut byte_iter = input.as_dangerous().iter();
    let write_byte = |w: &mut W, b: u8| {
        if show_ascii && b.is_ascii_graphic() {
            w.write_char('\'')?;
            w.write_char(b as char)?;
            w.write_char('\'')
        } else {
            write!(w, "{:x}", b)
        }
    };
    if let Some(byte) = byte_iter.next() {
        write_byte(w, *byte)?;
    }
    for byte in byte_iter {
        w.write_char(' ')?;
        write_byte(w, *byte)?;
    }
    Ok(())
}

fn write_str_contents_head<W>(w: &mut W, input: &str, max: usize) -> Result<bool, fmt::Error>
where
    W: Write,
{
    let mut i = 0;
    let mut iter = input.chars();
    while let Some(c) = iter.next() {
        i += 1;
        w.write_char(c)?;
        if i == max {
            return Ok(iter.next().is_some());
        }
    }
    Ok(false)
}

fn write_str_contents_tail<W>(w: &mut W, input: &str, skip: usize) -> fmt::Result
where
    W: Write,
{
    for c in input.chars().skip(skip) {
        w.write_char(c)?;
    }
    Ok(())
}

fn head_tail_max(max: usize) -> (usize, usize) {
    let half = max / 2;
    (half + max % 2, half)
}
