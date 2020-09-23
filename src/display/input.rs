use core::fmt::{self, Write};
use core::result::Result;
use core::str;

use unicode_width::UnicodeWidthChar;

use crate::error::Invalid;
use crate::input::{input, CharIter, Input};

const DEFAULT_SECTION: Section<'static> = Section::HeadTail { max: 1024 };
const DEFAULT_COLUMN_WIDTH: usize = 140;

// (ie ' ..' or '.. ')
const HAS_MORE_LEN: usize = 3;

fn init_column_width_for_input(column_width: Option<usize>) -> usize {
    let column_width = column_width.unwrap_or(DEFAULT_COLUMN_WIDTH);
    // for [] or ""
    column_width.saturating_sub(2)
}

pub(crate) struct DisplayComputed<'i> {
    input: &'i Input,
    section: &'i Input,
    section_is_str: bool,
    span: Option<&'i Input>,
}

impl<'i> DisplayComputed<'i> {
    pub(crate) fn from_head(input: &'i Input, column_width: Option<usize>, str_hint: bool) -> Self {
        let column_width = init_column_width_for_input(column_width);
        if str_hint {
            let (section, section_is_str) = take_str_head_column_width(input, column_width, false);
            Self {
                input,
                section,
                section_is_str,
                span: None,
            }
        } else {
            let (head, _) = input.split_max(column_width);
            Self::from_bytes(input, column_width)
        }
        unimplemented!()
    }

    pub(crate) fn from_tail(input: &'i Input, column_width: Option<usize>, str_hint: bool) -> Self {
        unimplemented!()
    }

    pub(crate) fn from_head_tail(
        input: &'i Input,
        column_width: Option<usize>,
        str_hint: bool,
    ) -> Self {
        unimplemented!()
    }

    pub(crate) fn from_span(
        input: &'i Input,
        span: &'i Input,
        column_width: Option<usize>,
        str_hint: bool,
    ) -> Self {
        unimplemented!()
    }

    fn from_maybe_str(input: &'i Input, maybe_str: &[u8], column_width: usize) -> Self {
        // if let Ok(s) = str::from_utf8(maybe_str) {
        // } else {
        // }
        unimplemented!()
    }

    fn has_more_before(&self) -> bool {
        unimplemented!()
    }

    fn has_more_after(&self) -> bool {
        unimplemented!()
    }

    fn highlight_before(&self) -> bool {
        unimplemented!()
    }

    fn highlight_after(&self) -> bool {
        unimplemented!()
    }

    fn highlight_open(&self) -> bool {
        unimplemented!()
    }

    fn highlight_close(&self) -> bool {
        unimplemented!()
    }
}

enum InputWriter<W: Write> {
    Raw(W),
    Span(W),
    Highlight(W),
}

impl<W> Write for InputWriter<W>
where
    W: Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self {
            Self::Raw(w) => w.write_str(s),
            Self::Span(w) => w.write_str(s),
            Self::Highlight(w) => {
                for _ in 0..s.len() {
                    w.write_char('^')?;
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
    // fn value(w: W) -> Self {

    // }

    // fn highlight(w: W) -> Self {
    //     Self {
    //         w,
    //         in_span: false,
    //         highlight: true,
    //     }
    // }

    fn enter_span(&mut self) {
        *self = match *self {
            Self::Raw(w) | Self::Span(w) => Self::Span(w),
            Self::Highlight(w) => Self::Highlight(w),
        };
    }

    fn leave_span(&mut self) {
        *self = match *self {
            Self::Raw(w) | Self::Span(w) => Self::Raw(w),
            Self::Highlight(w) => Self::Highlight(w),
        };
    }

    fn write_delim(&mut self, delim: char, highlight: bool) -> fmt::Result {
        match *self {
            Self::Raw(w) | Self::Span(w) => w.write_char(delim),
            Self::Highlight(w) if highlight => w.write_char('^'),
            Self::Highlight(w) => w.write_char(' '),
        }
    }

    fn write_space(&mut self) -> fmt::Result {
        match *self {
            Self::Raw(w) | Self::Span(w) | Self::Highlight(w) => w.write_char(' '),
        }
    }
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
    pub fn from_formatter(input: &'i Input, f: &fmt::Formatter<'_>) -> Self {
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

    // TODO
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
    pub fn span(mut self, span: &'i Input, max: usize) -> Self {
        self.section = Some(Section::Span { span, max });
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
        let writer = InputWriter::Raw(w);
        if self.str_hint {
            if let Ok(s) = str::from_utf8(self.input.as_dangerous()) {
                // let writer = InputWriter::
                write_str(w, s, self.section)
            } else {
                write_bytes(&mut writer, self.input, true, self.section)
            }
        } else {
            write_bytes(&mut writer, self.input, false, self.section)
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

fn elements_to_fit_column_width<'i, T, F>(
    mut acc: T,
    column_width: usize,
    space_separated: bool,
    mut next_element: F,
) -> T
where
    F: FnMut(&T) -> Option<(T, usize, bool)>,
{
    debug_assert!(
        column_width >= HAS_MORE_LEN,
        "should have enough space for at least one has more"
    );
    let mut is_first = true;
    let mut budget = column_width;
    loop {
        if let Some((next_acc, display_width, has_next)) = next_element(&acc) {
            // Make sure we have room for the seperator if any
            let element_cost = if space_separated {
                if is_first {
                    is_first = false;
                    display_width
                } else {
                    display_width + 1
                }
            } else {
                display_width
            };
            // Make sure we have room for the has more
            let required = if has_next {
                element_cost + HAS_MORE_LEN
            } else {
                element_cost
            };
            // Check that we have enough room for the this element,
            // and if so subtract it.
            if let Some(value) = budget.checked_sub(required) {
                acc = next_acc;
                // We did, update the budget.
                budget = value;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    acc
}

fn take_byte_str_head_column_width(complete: &Input, column_width: usize) -> &Input {
    let bytes = complete.as_dangerous();
    let mut byte_iter = bytes.iter();
    let offset = elements_to_fit_column_width(0, column_width, true, |acc| {
        byte_iter.next().map(|byte| {
            let cost = if byte.is_ascii_graphic() {
                b"'x'".len()
            } else {
                b"ff".len()
            };
            (acc + 1, cost, byte_iter.as_slice().len() > 0)
        })
    });
    input(&bytes[offset..])
}

fn take_str_head_column_width(
    complete: &Input,
    mut column_width: usize,
    cjk: bool,
) -> (&Input, bool) {
    let bytes = complete.as_dangerous();
    let mut is_str = true;
    let mut char_iter = CharIter::<Invalid>::new(complete);
    let offset = elements_to_fit_column_width(0, column_width, true, |acc| {
        char_iter.next().map(|result| {
            if let Ok(chr) = result {
                let display_width = if cjk {
                    chr.width_cjk().unwrap_or(1)
                } else {
                    chr.width().unwrap_or(1)
                };
                Some((display_width, ))
            } else {
                is_str = false;
                None
            }
        })
    });
    (input(&bytes[offset..]), is_str)
}

#[derive(Copy, Clone)]
enum Section<'i> {
    Tail { max: usize },
    Head { max: usize },
    HeadTail { max: usize },
    Span { span: &'i Input, max: usize },
}

fn write_str<W>(w: &mut W, input: &str, section: Option<Section<'_>>) -> fmt::Result
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
        Some(Section::Span { .. }) => unimplemented!(),
    }
}

fn write_bytes<W>(
    w: &mut InputWriter<W>,
    input: &Input,
    show_ascii: bool,
    section: Option<Section<'_>>,
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
        Some(Section::Span { .. }) => unimplemented!(),
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
            write!(w, "{:0>2x}", b)
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
