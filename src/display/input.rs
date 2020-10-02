use core::fmt::{self, Write};

use crate::display::{PreferredFormat, Section, SectionOpt};
use crate::input::Input;

// TODO: const DEFAULT_COLUMN_WIDTH: usize = 140;
const DEFAULT_SECTION_OPTION: SectionOpt<'static> = SectionOpt::HeadTail { width: 1024 };

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
    section: Option<Section<'i>>,
    section_opt: SectionOpt<'i>,
}

impl<'i> InputDisplay<'i> {
    /// Create a new `InputDisplay` given [`Input`].
    pub const fn new(input: &'i Input) -> Self {
        Self {
            input,
            str_hint: false,
            section: None,
            section_opt: DEFAULT_SECTION_OPTION,
        }
    }

    /// Derive an `InputDisplay` from a [`fmt::Formatter`] with defaults.
    ///
    /// - Precision (eg. `{:.2}`) formatting sets the element limit.
    /// - Alternate/pretty (eg. `{:#}`) formatting enables the UTF-8 hint.
    pub fn from_formatter(input: &'i Input, f: &fmt::Formatter<'_>) -> Self {
        let format = Self::new(input).str_hint(f.alternate());
        match f.width() {
            Some(width) => format.head_tail(width),
            None => format,
        }
    }

    /// Hint to the formatter that the [`Input`] is a UTF-8 `str`.
    pub fn str_hint(mut self, value: bool) -> Self {
        self.section = None;
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
    pub fn head_tail(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::HeadTail { width };
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
    pub fn head(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Head { width };
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
    pub fn tail(mut self, width: usize) -> Self {
        self.section = None;
        self.section_opt = SectionOpt::Tail { width };
        self
    }

    /// TODO: doc
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

    /// TODO
    pub fn prepare(&mut self) {
        // TODO: format
        let format = PreferredFormat::Str;
        let computed = self.section_opt.compute(self.input.as_dangerous(), format);
        self.section = Some(computed);
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
        match &self.section {
            None => {
                let mut this = self.clone();
                this.prepare();
                this.write(w)
            }
            Some(section) => section.write(w),
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
