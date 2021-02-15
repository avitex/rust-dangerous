use crate::error::{self, Context};
use crate::fmt::{self, Write};
use crate::input::{Bytes, Input, Private};

use super::{DisplayBase, InputDisplay, PreferredFormat};

const DEFAULT_MAX_WIDTH: usize = 80;
const INVALID_SPAN_ERROR: &str = "\
note: error span is not within the error input indicating the
      concrete error being used has a bug. Consider raising an
      issue with the maintainer!
";

/// Provides configurable [`error::Details`] formatting.
#[derive(Clone)]
#[must_use = "error displays must be written"]
pub struct ErrorDisplay<'a, T> {
    error: &'a T,
    banner: bool,
    format: Option<PreferredFormat>,
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
            format: None,
            banner: false,
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
        if self.error.input().is_string() || value {
            self.format(PreferredFormat::Str)
        } else {
            self.format(PreferredFormat::Bytes)
        }
    }

    /// Set the preferred way to format the [`Input`].
    pub fn format(mut self, format: PreferredFormat) -> Self {
        self.format = Some(format);
        self
    }

    fn write_sections(&self, w: &mut dyn Write) -> fmt::Result {
        let input = self.error.input();
        let root = self.error.backtrace().root();
        // Write description
        w.write_str("error attempting to ")?;
        root.operation().description(w)?;
        w.write_str(": ")?;
        self.error.description(w)?;
        w.write_char('\n')?;
        // Write inputs
        let input_display = self.configure_input_display(input.display());
        let format = input_display.get_format();
        let input = input.into_bytes();
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.configure_input_display(expected_value.display());
            w.write_str("expected:\n")?;
            write_input(w, expected_display, false)?;
            w.write_str("in:\n")?;
        }
        if root.span.is_within(input.span()) {
            write_input(w, input_display.span(root.span, self.input_max_width), true)?;
        } else {
            w.write_str(INVALID_SPAN_ERROR)?;
            w.write_str("input:\n")?;
            write_input(w, input_display, false)?;
        }
        // Write additional
        w.write_str("additional:\n  ")?;
        if let Some(span_range) = root.span.range_of(input.span()) {
            match format {
                PreferredFormat::Str | PreferredFormat::StrCjk | PreferredFormat::BytesAscii => {
                    w.write_str("error line: ")?;
                    w.write_usize(line_offset(&input, span_range.start))?;
                    w.write_str(", ")?;
                }
                _ => (),
            }
            w.write_str("error offset: ")?;
            w.write_usize(span_range.start)?;
            w.write_str(", input length: ")?;
            w.write_usize(input.len())?;
        } else {
            w.write_str("error: ")?;
            DisplayBase::fmt(&root.span, w)?;
            w.write_str("input: ")?;
            DisplayBase::fmt(&input.span(), w)?;
        }
        w.write_char('\n')?;
        // Write context backtrace
        w.write_str("backtrace:")?;
        let mut child_index = 1;
        let mut last_parent_depth = 0;
        let write_success = self.error.backtrace().walk(&mut |parent_depth, context| {
            let mut write = || {
                w.write_str("\n  ")?;
                if parent_depth == last_parent_depth {
                    w.write_str("  ")?;
                    w.write_usize(child_index)?;
                    child_index += 1;
                } else {
                    child_index = 1;
                    last_parent_depth = parent_depth;
                    w.write_usize(parent_depth)?;
                }
                w.write_str(". `")?;
                context.operation().description(w)?;
                w.write_char('`')?;
                if context.has_expected() {
                    w.write_str(" (expected ")?;
                    context.expected(w)?;
                    w.write_char(')')?;
                }
                fmt::Result::Ok(())
            };
            write().is_ok()
        });
        if write_success {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    fn configure_input_display<'b>(&self, display: InputDisplay<'b>) -> InputDisplay<'b> {
        if let Some(format) = self.format {
            display.format(format)
        } else {
            display
        }
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
        fmt::DisplayBase::fmt(&input.underline(), w)?;
        w.write_char('\n')?;
    }
    Ok(())
}
