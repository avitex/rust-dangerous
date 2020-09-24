use core::fmt::{self, Write};
use core::str;

use crate::display::{Section, SectionOption, SectionPart};
use crate::input::Input;

const DEFAULT_COLUMN_WIDTH: usize = 140;
const DEFAULT_SECTION_OPTION: SectionOption<'static> = SectionOption::HeadTail { width: 1024 };

const CHAR_DELIM: char = '\'';
const UNDERLINE: char = '^';
const HAS_MORE: &str = "..";
const HAS_MORE_IGNORED: &str = "  ";
const HAS_MORE_UNDERLINE: &str = "^^";
const STR_DELIM: char = '"';
const BYTES_DELIM_OPEN: char = '[';
const BYTES_DELIM_CLOSE: char = ']';

// (ie ' ..' or '.. ')
const HAS_MORE_LEN: usize = 3;
// (ie '[]' or '""')
const DELIM_LEN: usize = 2;

fn prepare_width(width: usize) -> usize {
    width.saturating_sub(DELIM_LEN)
}

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
    section_opt: Option<SectionOption<'i>>,
}

impl<'i> InputDisplay<'i> {
    /// Create a new `InputDisplay` given [`Input`].
    pub const fn new(input: &'i Input) -> Self {
        Self {
            input,
            str_hint: false,
            section: None,
            section_opt: Some(DEFAULT_SECTION_OPTION),
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
        self.section_opt = Some(SectionOption::HeadTail {
            width: prepare_width(width),
        });
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
        self.section_opt = Some(SectionOption::Head {
            width: prepare_width(width),
        });
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
        self.section_opt = Some(SectionOption::Tail {
            width: prepare_width(width),
        });
        self
    }

    /// TODO
    pub fn span(mut self, span: &'i Input, width: usize) -> Self {
        self.section = None;
        self.section_opt = Some(SectionOption::Span {
            span,
            width: prepare_width(width),
        });
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
        self.section_opt = None;
        self
    }

    /// TODO
    pub fn prepare(&mut self) {
        self.section = self
            .section_opt
            .map(|opt| Section::compute(opt, self.input, self.str_hint))
    }

    /// Writes the [`Input`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, mut w: W) -> fmt::Result
    where
        W: Write,
    {
        // If no section option is specified, just print everything fast.
        // Else if a section exists and it has been computed, print it.
        // Else compute the section it and print it.
        if self.section_opt.is_none() {
            let mut writer = InputWriter::new(w);
            if self.str_hint {
                if let Ok(input_str) = str::from_utf8(self.input.as_dangerous()) {
                    writer.write_delim(STR_DELIM, false)?;
                    writer.write_str(input_str)?;
                    writer.write_delim(STR_DELIM, false)
                } else {
                    writer.write_delim(BYTES_DELIM_OPEN, false)?;
                    write_bytes_contents(&mut writer, self.input.as_dangerous(), true)?;
                    writer.write_delim(BYTES_DELIM_CLOSE, false)
                }
            } else {
                writer.write_delim(BYTES_DELIM_OPEN, false)?;
                write_bytes_contents(&mut writer, self.input.as_dangerous(), false)?;
                writer.write_delim(BYTES_DELIM_CLOSE, false)
            }
        } else if let Some(ref section) = self.section {
            let mut writer = InputWriter::new(&mut w);
            write_section(&mut writer, section, self.str_hint)?;
            w.write_char('\n')?;
            let mut writer = InputWriter::underline(&mut w);
            write_section(&mut writer, section, self.str_hint)
        } else {
            let mut this = self.clone();
            this.prepare();
            this.write(w)
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

enum InputWriterState {
    Raw,
    Span,
    Underline,
}

struct InputWriter<W: Write>(W, InputWriterState);

impl<W> Write for InputWriter<W>
where
    W: Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.1 {
            InputWriterState::Raw | InputWriterState::Span => self.0.write_str(s),
            InputWriterState::Underline => {
                for _ in 0..s.len() {
                    self.0.write_char(UNDERLINE)?;
                }
                Ok(())
            }
        }
    }
}

impl<W> InputWriter<W>
where
    W: Write,
{
    fn new(w: W) -> Self {
        Self(w, InputWriterState::Raw)
    }

    fn underline(w: W) -> Self {
        Self(w, InputWriterState::Underline)
    }

    fn enter_span(&mut self) {
        self.1 = match self.1 {
            InputWriterState::Raw | InputWriterState::Span => InputWriterState::Span,
            InputWriterState::Underline => InputWriterState::Underline,
        };
    }

    fn leave_span(&mut self) {
        self.1 = match self.1 {
            InputWriterState::Raw | InputWriterState::Span => InputWriterState::Raw,
            InputWriterState::Underline => InputWriterState::Underline,
        };
    }

    fn write_delim(&mut self, delim: char, highlight: bool) -> fmt::Result {
        match self.1 {
            InputWriterState::Raw | InputWriterState::Span => self.0.write_char(delim),
            InputWriterState::Underline if highlight => self.0.write_char(UNDERLINE),
            InputWriterState::Underline => self.write_space(),
        }
    }

    fn write_space(&mut self) -> fmt::Result {
        self.0.write_char(' ')
    }

    fn write_more(&mut self, highlight: bool) -> fmt::Result {
        match self.1 {
            InputWriterState::Raw | InputWriterState::Span => self.0.write_str(HAS_MORE),
            InputWriterState::Underline if highlight => self.0.write_str(HAS_MORE_UNDERLINE),
            InputWriterState::Underline => self.0.write_str(HAS_MORE_IGNORED),
        }
    }
}

fn write_section<W>(w: &mut InputWriter<W>, section: &Section<'_>, show_ascii: bool) -> fmt::Result
where
    W: Write,
{
    if section.is_str() {
        if section.has_more_before() {
            w.write_more(section.highlight_before())?;
            w.write_space()?;
        }
        w.write_delim(STR_DELIM, section.highlight_open())?;
        for part in section.parts() {
            match part {
                SectionPart::Input(bytes) => {
                    w.write_str(str::from_utf8(bytes).unwrap())?;
                }
                SectionPart::Span(bytes) => {
                    w.enter_span();
                    w.write_str(str::from_utf8(bytes).unwrap())?;
                    w.leave_span();
                }
            }
        }
        w.write_delim(STR_DELIM, section.highlight_close())?;
        if section.has_more_after() {
            w.write_space()?;
            w.write_more(section.highlight_after())?;
        }
        Ok(())
    } else {
        w.write_delim(BYTES_DELIM_OPEN, section.highlight_open())?;
        if section.has_more_before() {
            w.write_more(section.highlight_before())?;
            w.write_space()?;
        }
        for part in section.parts() {
            match part {
                SectionPart::Input(bytes) => {
                    write_bytes_contents(w, bytes, show_ascii)?;
                }
                SectionPart::Span(bytes) => {
                    w.enter_span();
                    write_bytes_contents(w, bytes, show_ascii)?;
                    w.leave_span();
                }
            }
        }
        if section.has_more_after() {
            w.write_space()?;
            w.write_more(section.highlight_after())?;
        }
        w.write_delim(BYTES_DELIM_CLOSE, section.highlight_close())
    }
}

fn write_bytes_contents<W>(w: &mut InputWriter<W>, bytes: &[u8], show_ascii: bool) -> fmt::Result
where
    W: Write,
{
    let mut byte_iter = bytes.iter();
    let write_byte = |w: &mut InputWriter<W>, b: u8| {
        if show_ascii && b.is_ascii_graphic() {
            w.write_char(CHAR_DELIM)?;
            w.write_char(b as char)?;
            w.write_char(CHAR_DELIM)
        } else {
            write!(w, "{:0>2x}", b)
        }
    };
    if let Some(byte) = byte_iter.next() {
        write_byte(w, *byte)?;
    }
    for byte in byte_iter {
        w.write_space()?;
        write_byte(w, *byte)?;
    }
    Ok(())
}
