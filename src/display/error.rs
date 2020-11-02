use core::fmt::{self, Write};

use crate::error::{self, Context};
use crate::input::Input;
use crate::util::slice_ptr_range;

use super::{InputDisplay, PreferredFormat, WithFormatter};

const DEFAULT_MAX_WIDTH: usize = 80;

/// Provides configurable [`error::Details`] formatting.
pub struct ErrorDisplay<'a, T> {
    error: &'a T,
    banner: bool,
    underline: bool,
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
            underline: true,
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

    /// If enabled (enabled by default), writes an underline for an input span.
    pub fn underline(mut self, value: bool) -> Self {
        self.underline = value;
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

    /// Writes the [`error::Details`] to a writer with the chosen format.
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
        self.write_additional(w)?;
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
        let input_display = self.input_display(&input);
        let span_display = self.input_display(&span);
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.input_display(&expected_value);
            writeln!(w, "expected:")?;
            write_input(w, expected_display, false)?;
            writeln!(w, "in:")?;
        }
        if span.is_within(&input) {
            write_input(w, input_display.span(&span, self.input_max_width), true)
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
        write!(w, "backtrace:")?;
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

    fn write_additional<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        let input = self.error.input();
        let span = self.error.span();
        write!(w, "additional:\n  ")?;
        if span.is_within(&input) {
            let input_bounds = slice_ptr_range(input.as_dangerous());
            let span_bounds = slice_ptr_range(self.error.span().as_dangerous());
            let span_offset = span_bounds.start as usize - input_bounds.start as usize;
            match self.format {
                PreferredFormat::Str | PreferredFormat::StrCjk | PreferredFormat::BytesAscii => {
                    write!(w, "error line: {}, ", line_offset(&input, span_offset))?;
                }
                _ => (),
            }
            writeln!(
                w,
                "error offset: {}, input length: {}",
                span_offset,
                input.len()
            )
        } else {
            writeln!(
                w,
                "span ptr: {:?}, span length: {}",
                span.as_dangerous().as_ptr(),
                span.len(),
            )?;
            writeln!(
                w,
                "input ptr: {:?}, input length: {}",
                input.as_dangerous().as_ptr(),
                input.len(),
            )
        }
    }

    fn input_display<'b>(&self, input: &Input<'b>) -> InputDisplay<'b> {
        input.display().format(self.format)
    }
}

impl<'a, 'i, T> Clone for ErrorDisplay<'a, T> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<'a, 'i, T> fmt::Debug for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

impl<'a, 'i, T> fmt::Display for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

fn line_offset(input: &Input<'_>, span_offset: usize) -> usize {
    let (before_span, _) = input.clone().split_at_opt(span_offset).unwrap();
    before_span.count(b'\n') + 1
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
