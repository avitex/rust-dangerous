use core::fmt::{self, Write};

use crate::input::Input;

use super::section::{Section, SectionOpt};

const DEFAULT_SECTION_OPTION: SectionOpt<'static> = SectionOpt::HeadTail { width: 1024 };

/// Preferred [`Input`] formats.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
/// let formatted = dangerous::input("heya ♥".as_bytes())
///     .display()
///     .head_tail(16)
///     .to_string();
/// assert_eq!(formatted, "[68 65 .. 99 a5]");
/// ```
#[derive(Clone)]
pub struct InputDisplay<'i> {
    input: &'i Input,
    underline: bool,
    format: PreferredFormat,
    section: Option<Section<'i>>,
    section_opt: SectionOpt<'i>,
}

impl<'i> InputDisplay<'i> {
    /// Create a new `InputDisplay` given [`Input`].
    pub const fn new(input: &'i Input) -> Self {
        Self {
            input,
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
    pub fn from_formatter(input: &'i Input, f: &fmt::Formatter<'_>) -> Self {
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
    /// let full = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    /// let input = dangerous::input(full);
    /// let span = dangerous::input(&full[5..]);
    /// let formatted = input.display().span(span, 16).to_string();
    ///
    /// assert_eq!(formatted, "[.. cc dd ee ff]");
    /// ```
    pub fn span(mut self, span: &'i Input, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Span {
            width,
            span: span.as_dangerous(),
        };
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
        self.section_opt = SectionOpt::Full;
        self
    }

    /// Compute the sections of input to display.
    pub fn prepare(&mut self) {
        let computed = self
            .section_opt
            .compute(self.input.as_dangerous(), self.format);
        self.section = Some(computed);
    }

    /// Writes the [`Input`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, w: W) -> fmt::Result
    where
        W: Write,
    {
        match &self.section {
            None => {
                let mut this = self.clone();
                this.prepare();
                this.write(w)
            }
            Some(section) => section.write(w, self.underline),
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
