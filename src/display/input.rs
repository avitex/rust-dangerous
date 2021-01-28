use crate::fmt::{self, Write};
use crate::input::{Input, PrivateExt};

use super::section::{Section, SectionOpt};
use super::unit::{byte_display_width, byte_display_write, char_display_width, char_display_write};

const DEFAULT_SECTION_OPTION: SectionOpt<'static> = SectionOpt::HeadTail { width: 1024 };

/// Preferred [`Input`] formats.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PreferredFormat {
    /// Prefer displaying as a UTF-8 str.
    Str,
    /// Prefer displaying as a UTF-8 str with Chinese, Japanese or Korean
    /// characters.
    StrCjk,
    /// Prefer displaying as plain bytes.
    Bytes,
    /// Prefer displaying as bytes with valid ASCII graphic characters.
    BytesAscii,
}

impl fmt::Debug for PreferredFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Str => "Str",
            Self::StrCjk => "StrCjk",
            Self::Bytes => "Bytes",
            Self::BytesAscii => "BytesAscii",
        };
        f.write_str(s)
    }
}

/// Provides configurable [`Input`] formatting.
///
/// - Defaults to formatting an [`Input`] to a max displayable width of `1024`.
/// - The minimum settable display width is `16`.
///
/// # Format string options
///
/// | Option      | `"heya ♥"`                  | `&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF b'a']` |
/// | ----------- | --------------------------- | -------------------------------------- |
/// | `"{}"`      | `[68 65 79 61 20 e2 99 a5]` | `[ff ff ff ff ff 61]`                  |
/// | `"{:#}"`    | `"heya ♥"`                  | `[ff ff ff ff ff 'a']`                 |
/// | `"{:.16}"`  | `[68 65 .. 99 a5]`          | `[ff ff .. ff 61]`                     |
/// | `"{:#.16}"` | `"heya ♥"`                  | `[ff ff .. 'a']`                       |
///
/// # Example
///
/// ```
/// use dangerous::Input;
///
/// let formatted = dangerous::input("heya ♥".as_bytes())
///     .display()
///     .head_tail(16)
///     .to_string();
/// assert_eq!(formatted, "[68 65 .. 99 a5]");
/// ```
#[derive(Clone)]
#[must_use = "input displays must be written"]
pub struct InputDisplay<'i> {
    input: &'i [u8],
    underline: bool,
    format: PreferredFormat,
    section: Option<Section<'i>>,
    section_opt: SectionOpt<'i>,
}

impl<'i> InputDisplay<'i> {
    /// Create a new `InputDisplay` given [`Input`].
    pub fn new(input: &impl Input<'i>) -> Self {
        Self {
            input: input.as_dangerous_bytes(),
            format: PreferredFormat::Bytes,
            underline: false,
            section: None,
            section_opt: DEFAULT_SECTION_OPTION,
        }
    }

    /// Derive an `InputDisplay` from a [`fmt::Formatter`] with defaults.
    ///
    /// - Precision (eg. `{:.16}`) formatting sets the element limit.
    /// - Alternate/pretty (eg. `{:#}`) formatting enables the UTF-8 hint.
    pub fn from_formatter(input: &impl Input<'i>, f: &fmt::Formatter<'_>) -> Self {
        let format = Self::new(input).str_hint(f.alternate());
        match f.precision() {
            Some(width) => format.head_tail(width),
            None => format,
        }
    }

    /// Print the input underline for any provided span.
    pub fn underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    /// Hint to the formatter that the [`Input`] is a UTF-8 `str`.
    pub fn str_hint(self, value: bool) -> Self {
        if value {
            self.format(PreferredFormat::Str)
        } else {
            self.format(PreferredFormat::Bytes)
        }
    }

    /// Set the preferred way to format the [`Input`].
    pub fn format(mut self, format: PreferredFormat) -> Self {
        self.section = None;
        self.format = format;
        self
    }

    /// Show a `width` of [`Input`] at the head of the input and at the tail.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().head_tail(16).to_string();
    ///
    /// assert_eq!(formatted, "[aa bb .. ee ff]");
    /// ```
    pub fn head_tail(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::HeadTail { width };
        self
    }

    /// Show a `width` of [`Input`] at the head of the input.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().head(16).to_string();
    ///
    /// assert_eq!(formatted, "[aa bb cc dd ..]");
    /// ```
    pub fn head(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Head { width };
        self
    }

    /// Show a `width` of [`Input`] at the tail of the input.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().tail(16).to_string();
    ///
    /// assert_eq!(formatted, "[.. cc dd ee ff]");
    /// ```
    pub fn tail(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Tail { width };
        self
    }

    /// Show a `width` of input [`Input`] targeting a span.
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let full = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    /// let input = dangerous::input(full);
    /// let span = dangerous::input(&full[5..]);
    /// let formatted = input.display().span(&span, 16).to_string();
    ///
    /// assert_eq!(formatted, "[.. cc dd ee ff]");
    /// ```
    pub fn span(mut self, span: &impl Input<'i>, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Span {
            width,
            span: span.as_dangerous_bytes(),
        };
        self
    }

    /// Shows the all of the elements in the [`Input`].
    ///
    /// # Example
    ///
    /// ```
    /// use dangerous::Input;
    ///
    /// let input = dangerous::input(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    /// let formatted = input.display().full().to_string();
    ///
    /// assert_eq!(formatted, "[aa bb cc dd ee ff]");
    /// ```
    pub fn full(mut self) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Full;
        self
    }

    /// Compute the sections of input to display.
    pub fn prepare(mut self) -> Self {
        let computed = self.section_opt.compute(self.input, self.format);
        self.section = Some(computed);
        self
    }
}

impl<'i> fmt::DisplayBase for InputDisplay<'i> {
    fn fmt(&self, w: &mut dyn Write) -> fmt::Result {
        match &self.section {
            None => self.clone().prepare().fmt(w),
            Some(section) => section.write(w, self.underline),
        }
    }
}

impl<'i> fmt::Debug for InputDisplay<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

impl<'i> fmt::Display for InputDisplay<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(super) struct InputWriter<'a> {
    w: &'a mut dyn Write,
    underline: bool,
    full: &'a [u8],
    span: Option<&'a [u8]>,
}

impl<'a> InputWriter<'a> {
    pub(super) fn new(
        w: &'a mut dyn Write,
        full: &'a [u8],
        span: Option<&'a [u8]>,
        underline: bool,
    ) -> Self {
        Self {
            w,
            full,
            span,
            underline,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Bytes

    pub(super) fn write_bytes_side(&mut self, side: &[u8], show_ascii: bool) -> fmt::Result {
        self.write_bytes_open(side)?;
        self.write_bytes(side, show_ascii)?;
        self.write_bytes_close(side)
    }

    pub(super) fn write_bytes_sides(
        &mut self,
        left: &[u8],
        right: &[u8],
        show_ascii: bool,
    ) -> fmt::Result {
        self.write_bytes_open(left)?;
        self.write_bytes(left, show_ascii)?;
        self.write_space(1)?;
        self.write_more(is_span_overlapping_end(left, self.span))?;
        self.write_space(1)?;
        self.write_bytes(right, show_ascii)?;
        self.write_bytes_close(right)
    }

    fn write_bytes_open(&mut self, bytes: &[u8]) -> fmt::Result {
        if has_more_before(bytes, self.full) {
            self.write_delim('[', false)?;
            self.write_more(is_span_overlapping_start(bytes, self.span))?;
            self.write_space(1)
        } else {
            self.write_delim('[', is_span_pointing_to_start(bytes, self.span))
        }
    }

    fn write_bytes_close(&mut self, bytes: &[u8]) -> fmt::Result {
        if has_more_after(bytes, self.full) {
            self.write_space(1)?;
            self.write_more(is_span_overlapping_end(bytes, self.span))?;
            self.write_delim(']', false)
        } else {
            self.write_delim(']', is_span_pointing_to_end(bytes, self.span))
        }
    }

    fn write_bytes(&mut self, bytes: &[u8], show_ascii: bool) -> fmt::Result {
        let mut iter = bytes.iter().copied();
        if let Some(byte) = iter.next() {
            self.write_byte(byte, bytes, show_ascii)?;
        }
        for (i, byte) in (1..bytes.len()).zip(iter) {
            self.write_space(1)?;
            self.write_byte(byte, &bytes[i..], show_ascii)?;
        }
        Ok(())
    }

    fn write_byte(&mut self, byte: u8, remaining: &[u8], show_ascii: bool) -> fmt::Result {
        if self.underline {
            let byte_display_width = byte_display_width(byte, show_ascii);
            if is_section_start_within_span(remaining, self.span) {
                self.write_underline(byte_display_width)
            } else {
                self.write_space(byte_display_width)
            }
        } else {
            byte_display_write(byte, show_ascii, self.w)
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Str

    pub(super) fn write_str_side(&mut self, side: &str, cjk: bool) -> fmt::Result {
        self.write_str_open(side)?;
        self.write_str(side, cjk)?;
        self.write_str_close(side)
    }

    pub(super) fn write_str_sides(&mut self, left: &str, right: &str, cjk: bool) -> fmt::Result {
        self.write_str_open(left)?;
        self.write_str(left, cjk)?;
        self.write_delim('"', false)?;
        self.write_space(1)?;
        self.write_more(is_span_overlapping_end(left.as_bytes(), self.span))?;
        self.write_space(1)?;
        self.write_delim('"', false)?;
        self.write_str(right, cjk)?;
        self.write_str_close(right)
    }

    fn write_str_open(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if has_more_before(bytes, self.full) {
            self.write_more(is_span_overlapping_start(bytes, self.span))?;
            self.write_space(1)?;
            self.write_delim('"', false)
        } else {
            self.write_delim('"', is_span_pointing_to_start(bytes, self.span))
        }
    }

    fn write_str_close(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if has_more_after(bytes, self.full) {
            self.write_delim('"', false)?;
            self.write_space(1)?;
            self.write_more(is_span_overlapping_end(bytes, self.span))
        } else {
            self.write_delim('"', is_span_pointing_to_end(bytes, self.span))
        }
    }

    fn write_str(&mut self, s: &str, cjk: bool) -> fmt::Result {
        let bytes = s.as_bytes();
        if self.underline {
            if is_span_start_within_section(bytes, self.span) {
                let mut offset = 0;
                for c in s.chars() {
                    let char_display_width = char_display_width(c, cjk);
                    if is_section_start_within_span(&bytes[offset..], self.span) {
                        self.write_underline(char_display_width)?;
                    } else {
                        self.write_space(char_display_width)?;
                    }
                    offset += c.len_utf8();
                }
            } else {
                for c in s.chars() {
                    self.write_space(char_display_width(c, cjk))?;
                }
            }
        } else {
            for c in s.chars() {
                char_display_write(c, self.w)?;
            }
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    // Private

    fn write_more(&mut self, highlight: bool) -> fmt::Result {
        if self.underline {
            if highlight {
                self.write_underline(2)
            } else {
                self.write_space(2)
            }
        } else {
            self.w.write_str("..")
        }
    }

    fn write_delim(&mut self, delim: char, highlighted: bool) -> fmt::Result {
        if self.underline {
            if highlighted {
                self.write_underline(1)
            } else {
                self.write_space(1)
            }
        } else {
            self.w.write_char(delim)
        }
    }

    fn write_space(&mut self, len: usize) -> fmt::Result {
        self.write_char_len(' ', len)
    }

    fn write_underline(&mut self, len: usize) -> fmt::Result {
        self.write_char_len('^', len)
    }

    fn write_char_len(&mut self, c: char, len: usize) -> fmt::Result {
        for _ in 0..len {
            self.w.write_char(c)?;
        }
        Ok(())
    }
}

fn has_more_before(bytes: &[u8], full: &[u8]) -> bool {
    let section_bounds = bytes.as_ptr_range();
    let full_bounds = full.as_ptr_range();
    section_bounds.start > full_bounds.start
}

fn has_more_after(bytes: &[u8], full: &[u8]) -> bool {
    let section_bounds = bytes.as_ptr_range();
    let full_bounds = full.as_ptr_range();
    section_bounds.end < full_bounds.end
}

fn is_span_start_within_section(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start <= span_bounds.start && section_bounds.end > span_bounds.start
    })
}

fn is_section_start_within_span(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start >= span_bounds.start && section_bounds.start < span_bounds.end
    })
}

fn is_span_overlapping_end(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.end < span_bounds.end
    })
}

fn is_span_overlapping_start(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start > span_bounds.start
    })
}

fn is_span_pointing_to_start(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        span.is_empty() && section_bounds.start == span_bounds.start
    })
}

fn is_span_pointing_to_end(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        span.is_empty() && section_bounds.end == span_bounds.end
    })
}
