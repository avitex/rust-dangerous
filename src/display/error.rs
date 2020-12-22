use crate::error::{self, Context, Details as ErrorDetails};
use crate::input::Input;

use super::{fmt, InputDisplay, PreferredFormat};

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
    pub fn from_formatter<F>(error: &'a T, f: &F) -> Self
    where
        F: fmt::FormatterBase + ?Sized,
    {
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

    fn write_sections<F>(&self, f: &mut F) -> fmt::Result
    where
        F: fmt::FormatterBase + ?Sized,
    {
        self.write_description(f)?;
        self.write_inputs(f)?;
        self.write_additional(f)?;
        self.write_context_backtrace(f)
    }

    fn write_description<F>(&self, f: &mut F) -> fmt::Result
    where
        F: fmt::FormatterBase + ?Sized,
    {
        f.write_str("error attempting to ")?;
        f.write_display(&self.error.context_stack().root().operation())?;
        f.write_str(": ")?;
        self.error.description(f.as_dyn_mut())?;
        f.write_char('\n')
    }

    fn write_inputs<F>(&self, f: &mut F) -> fmt::Result
    where
        F: fmt::FormatterBase + ?Sized,
    {
        let input = self.error.input();
        let span = self.error.span();
        let input_display = self.input_display(&input);
        let span_display = self.input_display(&span);
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.input_display(&expected_value);
            f.write_str("expected:\n")?;
            write_input(f, expected_display, false)?;
            f.write_str("in:\n")?;
        }
        if span.is_within(&input) {
            write_input(f, input_display.span(&span, self.input_max_width), true)
        } else {
            f.write_str(concat!(
                "note: error span is not within the error input indicating the\n",
                "      concrete error being used has a bug. Consider raising an\n",
                "      issue with the maintainer!\n",
            ))?;
            f.write_str("span: \n")?;
            write_input(f, span_display, false)?;
            f.write_str("input: \n")?;
            write_input(f, input_display, false)
        }
    }

    fn write_context_backtrace<F>(&self, f: &mut F) -> fmt::Result
    where
        F: fmt::FormatterBase + ?Sized,
    {
        f.write_str("backtrace:")?;
        let write_success = self.error.context_stack().walk(&mut |i, c| {
            fn write_context<F: fmt::FormatterBase + ?Sized>(
                f: &mut F,
                i: usize,
                c: &dyn Context,
            ) -> fmt::Result {
                f.write_str("\n ")?;
                f.write_usize(i)?;
                f.write_str(". `")?;
                f.write_str(c.operation())?;
                f.write_char('`')?;
                if let Some(expected) = c.expected() {
                    f.write_str(" (expected ")?;
                    f.write_display(expected)?;
                    f.write_char(')')?;
                }
                Ok(())
            }
            write_context(f, i, c).is_ok()
        });
        if write_success {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    fn write_additional<F>(&self, f: &mut F) -> fmt::Result
    where
        F: fmt::FormatterBase + ?Sized,
    {
        let input = self.error.input();
        let span = self.error.span();
        f.write_str("additional:\n  ")?;
        if span.is_within(&input) {
            let input_bounds = input.as_dangerous().as_ptr_range();
            let span_bounds = self.error.span().as_dangerous().as_ptr_range();
            let span_offset = span_bounds.start as usize - input_bounds.start as usize;
            match self.format {
                PreferredFormat::Str | PreferredFormat::StrCjk | PreferredFormat::BytesAscii => {
                    f.write_str("error line: ")?;
                    f.write_usize(line_offset(&input, span_offset))?;
                    f.write_str(", ")?;
                }
                _ => (),
            }
            f.write_str("error offset: ")?;
            f.write_usize(span_offset)?;
            f.write_str(", input length: ")?;
            f.write_usize(input.len())?;
        } else {
            f.write_str("span ptr: ")?;
            f.write_usize(span.as_dangerous().as_ptr() as usize)?;
            f.write_str(", span length: ")?;
            f.write_usize(span.len())?;
            f.write_str("input ptr: ")?;
            f.write_usize(input.as_dangerous().as_ptr() as usize)?;
            f.write_str(", input length: ")?;
            f.write_usize(input.len())?;
        }
        f.write_char('\n')
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

impl<'a, 'i, T> fmt::DisplayBase for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut dyn fmt::FormatterBase) -> fmt::Result {
        if self.banner {
            f.write_str("\n-- INPUT ERROR ---------------------------------------------\n")?;
            self.write_sections(f)?;
            f.write_str("\n------------------------------------------------------------\n")
        } else {
            self.write_sections(f)
        }
    }
}

forward_fmt!(impl<'a, 'i, T> Display for ErrorDisplay<'a, T> where T: ErrorDetails<'i>);

impl<'a, 'i, T> fmt::DebugBase for ErrorDisplay<'a, T>
where
    T: error::Details<'i>,
{
    fn fmt(&self, f: &mut dyn fmt::FormatterBase) -> fmt::Result {
        fmt::DisplayBase::fmt(self, f)
    }
}

forward_fmt!(impl<'a, 'i, T> Debug for ErrorDisplay<'a, T> where T: ErrorDetails<'i>);

fn line_offset(input: &Input<'_>, span_offset: usize) -> usize {
    let (before_span, _) = input.clone().split_at_opt(span_offset).unwrap();
    before_span.count(b'\n') + 1
}

fn write_input<F>(f: &mut F, mut input: InputDisplay<'_>, underline: bool) -> fmt::Result
where
    F: fmt::FormatterBase + ?Sized,
{
    input.prepare();
    f.write_str("> ")?;
    fmt::DisplayBase::fmt(&input, f.as_dyn_mut())?;
    f.write_char('\n')?;
    if underline {
        f.write_str("  ")?;
        fmt::DisplayBase::fmt(&input.underline(true), f.as_dyn_mut())?;
        f.write_char('\n')?;
    }
    Ok(())
}
