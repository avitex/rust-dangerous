use crate::error::{self, Context};
use crate::fmt::{self, Write};
use crate::input::{Bytes, Input, Private};

use super::{InputDisplay, PreferredFormat};

const DEFAULT_MAX_WIDTH: usize = 80;

/// Provides configurable [`error::Details`] formatting.
#[must_use = "error displays must be written"]
pub struct ErrorDisplay<'a, T> {
    error: &'a T,
    banner: bool,
    format: PreferredFormat,
    input_max_width: usize,
}

impl<'a, 'i, T> ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    /// Create a new `ErrorDisplay` given [`error::Details`].
    pub fn new(error: &'a T) -> Self {
        Self {
            error,
            banner: false,
            format: PreferredFormat::Bytes,
            input_max_width: DEFAULT_MAX_WIDTH,
        }
    }

    /// Derive an `ErrorDisplay` from a [`fmt::Formatter`] with defaults.
    pub fn from_formatter(error: &'a T, f: &fmt::Formatter<'_>) -> Self {
        Self::new(error).str_hint(f.alternate())
    }

    /// Set whether or not a banner should printed around the error.
    pub fn banner(mut self, value: bool) -> Self {
        self.banner = value;
        self
    }

    /// Set the `max-width` for wrapping error output.
    pub fn input_max_width(mut self, value: usize) -> Self {
        self.input_max_width = value;
        self
    }

    /// Hint to the formatter that the [`crate::Input`] is a UTF-8 `str`.
    pub fn str_hint(self, value: bool) -> Self {
        if value {
            self.format(PreferredFormat::Str)
        } else {
            self.format(PreferredFormat::Bytes)
        }
    }

    /// Set the preferred way to format the [`Input`].
    pub fn format(mut self, format: PreferredFormat) -> Self {
        self.format = format;
        self
    }

    fn write_sections(&self, w: &mut dyn Write) -> fmt::Result {
        let input = self.error.input();
        let span = self.error.span();
        // Write description
        w.write_str("error attempting to ")?;
        w.write_str(self.error.context_stack().root().operation())?;
        w.write_str(": ")?;
        self.error.description(w)?;
        w.write_char('\n')?;
        // Write inputs
        let input_display = self.configure_input_display(input.display());
        let span_display = self.configure_input_display(span.display());
        let input = input.into_bytes();
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.configure_input_display(expected_value.display());
            w.write_str("expected:\n")?;
            write_input(w, expected_display, false)?;
            w.write_str("in:\n")?;
        }
        if span.is_within(&input) {
            write_input(w, input_display.span(&span, self.input_max_width), true)?;
        } else {
            w.write_str(concat!(
                "note: error span is not within the error input indicating the\n",
                "      concrete error being used has a bug. Consider raising an\n",
                "      issue with the maintainer!\n",
            ))?;
            w.write_str("span:\n")?;
            write_input(w, span_display, false)?;
            w.write_str("input:\n")?;
            write_input(w, input_display, false)?;
        }
        // Write additional
        w.write_str("additional:\n  ")?;
        if span.is_within(&input) {
            let input_bounds = input.as_dangerous().as_ptr_range();
            let span_bounds = span.as_dangerous().as_ptr_range();
            let span_offset = span_bounds.start as usize - input_bounds.start as usize;
            match self.format {
                PreferredFormat::Str | PreferredFormat::StrCjk | PreferredFormat::BytesAscii => {
                    w.write_str("error line: ")?;
                    w.write_usize(line_offset(&input, span_offset))?;
                    w.write_str(", ")?;
                }
                _ => (),
            }
            w.write_str("error offset: ")?;
            w.write_usize(span_offset)?;
            w.write_str(", input length: ")?;
            w.write_usize(input.len())?;
        } else {
            w.write_str("span ptr: ")?;
            w.write_usize(span.as_dangerous().as_ptr() as usize)?;
            w.write_str(", span length: ")?;
            w.write_usize(span.len())?;
            w.write_str("input ptr: ")?;
            w.write_usize(input.as_dangerous().as_ptr() as usize)?;
            w.write_str(", input length: ")?;
            w.write_usize(input.len())?;
        }
        w.write_char('\n')?;
        // Write context backtrace
        w.write_str("backtrace:")?;
        let write_success = self.error.context_stack().walk(&mut |i, c| {
            let writer = |w: &mut dyn Write, i, c: &dyn Context| {
                w.write_str("\n  ")?;
                w.write_usize(i)?;
                w.write_str(". `")?;
                w.write_str(c.operation())?;
                w.write_char('`')?;
                if c.has_expected() {
                    w.write_str(" (expected ")?;
                    c.expected(w)?;
                    w.write_char(')')?;
                }
                fmt::Result::Ok(())
            };
            writer(w, i, c).is_ok()
        });
        if write_success {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    fn configure_input_display<'b>(&self, display: InputDisplay<'b>) -> InputDisplay<'b> {
        display.format(self.format)
    }
}

impl<'a, 'i, T> Clone for ErrorDisplay<'a, T> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<'a, 'i, T> fmt::DisplayBase for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, w: &mut dyn Write) -> fmt::Result {
        if self.banner {
            w.write_str("\n-- INPUT ERROR ---------------------------------------------\n")?;
            self.write_sections(w)?;
            w.write_str("\n------------------------------------------------------------\n")
        } else {
            self.write_sections(w)
        }
    }
}

impl<'a, 'i, T> fmt::Debug for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

impl<'a, 'i, T> fmt::Display for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

fn line_offset(input: &Bytes<'_>, span_offset: usize) -> usize {
    match input.clone().split_at_opt(span_offset) {
        Some((before_span, _)) => before_span.count(b'\n') + 1,
        // Will never be reached in practical usage but we handle to avoid
        // unwrapping.
        None => 0,
    }
}

fn write_input(w: &mut dyn Write, input: InputDisplay<'_>, underline: bool) -> fmt::Result {
    let input = input.prepare();
    w.write_str("> ")?;
    fmt::DisplayBase::fmt(&input, w)?;
    w.write_char('\n')?;
    if underline {
        w.write_str("  ")?;
        fmt::DisplayBase::fmt(&input.underline(true), w)?;
        w.write_char('\n')?;
    }
    Ok(())
}
