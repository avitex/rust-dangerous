use core::fmt::{self, Write};

use crate::error::{Context, ErrorDetails};
use crate::input::Input;

use super::{InputDisplay, PreferredFormat, WithFormatter};

const DEFAULT_MAX_WIDTH: usize = 80;

/// Provides configurable [`ErrorDetails`] formatting.
#[derive(Clone)]
pub struct ErrorDisplay<'a, T> {
    error: &'a T,
    banner: bool,
    underline: bool,
    format: PreferredFormat,
    max_width: Option<usize>,
}

impl<'a, 'i, T> ErrorDisplay<'a, T>
where
    T: ErrorDetails<'i>,
{
    /// Create a new `ErrorDisplay` given an [`ErrorDetails`].
    pub fn new(error: &'a T) -> Self {
        Self {
            error,
            banner: false,
            underline: true,
            format: PreferredFormat::Bytes,
            max_width: Some(DEFAULT_MAX_WIDTH),
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

    /// If enabled (enabled by default), writes an underline for an input span.
    pub fn underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    /// Set the `max-width` for wrapping error output.
    pub fn max_width(mut self, value: Option<usize>) -> Self {
        self.max_width = value;
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

    /// Writes the [`ErrorDetails`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, mut w: W) -> fmt::Result
    where
        W: Write,
    {
        if self.banner {
            w.write_str("\n-- INPUT ERROR ---------------------------------------------\n")?;
            self.write_sections(&mut w)?;
            w.write_str("\n------------------------------------------------------------\n")
        } else {
            self.write_sections(&mut w)
        }
    }

    fn write_sections<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        self.write_description(w)?;
        self.write_inputs(w)?;
        self.write_context_backtrace(w)
    }

    fn write_description<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        writeln!(
            w,
            "error attempting to {}: {}",
            self.error.context_stack().root().operation(),
            WithFormatter(|f| self.error.description(f)),
        )
    }

    fn write_inputs<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        let input = self.error.input();
        let span = self.error.span();
        let input_display = self.input_display(input);
        let span_display = self.input_display(span);
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.input_display(expected_value);
            writeln!(w, "expected:")?;
            write_input(w, expected_display, false)?;
        }
        if span.is_within(input) {
            write_input(w, input_display.span(span, 40), true)
        } else {
            writeln!(
                w,
                concat!(
                    "note: error span is not within the error input indicating the\n",
                    "      concrete error being used has a bug. Consider raising an\n",
                    "      issue with the maintainer!",
                )
            )?;
            writeln!(w, "span: ")?;
            write_input(w, span_display, false)?;
            writeln!(w, "input: ")?;
            write_input(w, input_display, false)
        }
    }

    fn write_context_backtrace<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        write!(w, "context bracktrace:")?;
        let write_success = self.error.context_stack().walk(&mut |i, c| {
            if write!(w, "\n  {}. `{}`", i, c.operation()).is_err() {
                return false;
            }
            if let Some(expected) = c.expected() {
                if write!(w, " (expected {})", expected).is_err() {
                    return false;
                }
            }
            true
        });
        if write_success {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    fn input_display<'b>(&self, input: &'b Input) -> InputDisplay<'b> {
        input.display().format(self.format)
    }
}

impl<'a, 'i, T> fmt::Debug for ErrorDisplay<'a, T>
where
    T: ErrorDetails<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

impl<'a, 'i, T> fmt::Display for ErrorDisplay<'a, T>
where
    T: ErrorDetails<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

fn write_input<W>(w: &mut W, mut input: InputDisplay<'_>, underline: bool) -> fmt::Result
where
    W: Write,
{
    input.prepare();
    writeln!(w, "> {}", input)?;
    if underline {
        writeln!(w, "  {}", input.underline(true))?;
    }
    Ok(())
}
