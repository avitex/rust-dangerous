use crate::error::{self, Context};
use crate::fmt::{self, Write};
use crate::input::{Bytes, Input};

use super::{ansi, DisplayBase, InputDisplay, PreferredFormat};

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
    color: bool,
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
        let format = if error.input().is_string() {
            PreferredFormat::Str
        } else {
            PreferredFormat::Bytes
        };
        Self {
            error,
            format,
            color: true,
            banner: false,
            input_max_width: DEFAULT_MAX_WIDTH,
        }
    }

    /// Derive an `ErrorDisplay` from a [`fmt::Formatter`] with defaults.
    pub fn from_formatter(error: &'a T, f: &fmt::Formatter<'_>) -> Self {
        if f.alternate() {
            Self::new(error).str_hint()
        } else {
            Self::new(error)
        }
    }

    /// Set whether or not the error should print with colors.
    pub fn color(mut self, value: bool) -> Self {
        self.color = value;
        self
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
    pub fn str_hint(self) -> Self {
        match self.format {
            PreferredFormat::Bytes | PreferredFormat::BytesAscii => {
                self.format(PreferredFormat::Str)
            }
            _ => self,
        }
    }

    /// Set the preferred way to format the [`Input`].
    pub fn format(mut self, format: PreferredFormat) -> Self {
        self.format = format;
        self
    }

    fn write_sections(&self, w: &mut dyn Write) -> fmt::Result {
        let input = self.error.input();
        let root = self.error.backtrace().root();
        // Write description
        self.write_ansi(w, ansi::STYLE_BOLD)?;
        self.write_ansi(w, ansi::FG_RED)?;
        w.write_str("failed to ")?;
        root.operation().description(w)?;
        w.write_str(": ")?;
        self.write_ansi(w, ansi::FG_DEFAULT)?;
        self.error.description(w)?;
        self.write_ansi(w, ansi::STYLE_END)?;
        w.write_str("\n\n")?;
        // Write inputs
        let input_display = self.configure_input_display(input.display());
        let input = input.into_bytes();
        if let Some(expected_value) = self.error.expected() {
            let expected_display = self.configure_input_display(expected_value.display());
            self.write_bold(w, "expected:\n")?;
            self.write_input(w, expected_display, false)?;
            self.write_bold(w, "in:\n")?;
        }
        if root.span.is_within(input.span()) {
            self.write_input(w, input_display.span(root.span, self.input_max_width), true)?;
        } else {
            w.write_str(INVALID_SPAN_ERROR)?;
            w.write_str("input:\n")?;
            self.write_input(w, input_display, false)?;
        }
        // Write additional
        self.write_bold(w, "additional:\n  ")?;
        if let Some(span_range) = root.span.range_of(input.span()) {
            if matches!(
                self.format,
                PreferredFormat::Str | PreferredFormat::StrCjk | PreferredFormat::BytesAscii
            ) {
                w.write_str("error line: ")?;
                self.write_usize(w, line_offset(&input, span_range.start))?;
                w.write_str(", ")?;
            }
            w.write_str("error offset: ")?;
            self.write_usize(w, span_range.start)?;
            w.write_str(", input length: ")?;
            self.write_usize(w, input.len())?;
        } else {
            w.write_str("error: ")?;
            DisplayBase::fmt(&root.span, w)?;
            w.write_str("input: ")?;
            DisplayBase::fmt(&input.span(), w)?;
        }
        w.write_char('\n')?;
        // Write context backtrace
        self.write_bold(w, "backtrace:")?;
        let mut child_index = 1;
        let mut last_parent_depth = 0;
        let write_success = self.error.backtrace().walk(&mut |parent_depth, context| {
            let mut write = || {
                w.write_str("\n  ")?;
                self.write_ansi(w, ansi::STYLE_DIM)?;
                if parent_depth == last_parent_depth {
                    w.write_str("  ")?;
                    w.write_usize(child_index)?;
                    child_index += 1;
                } else {
                    child_index = 1;
                    last_parent_depth = parent_depth;
                    w.write_usize(parent_depth)?;
                }
                w.write_str(". ")?;
                self.write_ansi(w, ansi::STYLE_END)?;
                //w.write_char('`')?;
                context.operation().description(w)?;
                //w.write_char('`')?;
                if context.has_expected() {
                    self.write_ansi(w, ansi::STYLE_BOLD)?;
                    self.write_ansi(w, ansi::FG_RED)?;
                    w.write_str(" (expected ")?;
                    context.expected(w)?;
                    w.write_char(')')?;
                    self.write_ansi(w, ansi::FG_DEFAULT)?;
                    self.write_ansi(w, ansi::STYLE_END)?;
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

    fn write_input(&self, w: &mut dyn Write, input: InputDisplay<'_>, underline: bool) -> fmt::Result {
        let input = input.prepare();
        self.write_dim(w, "  ")?;
        fmt::DisplayBase::fmt(&input, w)?;
        w.write_char('\n')?;
        if underline {
            w.write_str("  ")?;
            fmt::DisplayBase::fmt(&input.underline(), w)?;
            w.write_char('\n')?;
        }
        Ok(())
    }  
    
    fn write_usize(&self, w: &mut dyn Write, value: usize) -> fmt::Result {
        self.write_ansi(w, ansi::STYLE_BOLD)?;
        self.write_ansi(w, ansi::FG_BLUE)?;
        w.write_usize(value)?;
        self.write_ansi(w, ansi::FG_DEFAULT)?;
        self.write_ansi(w, ansi::STYLE_END)
    }

    fn write_dim(&self, w: &mut dyn Write, s: &str) -> fmt::Result {
        self.write_ansi(w, ansi::STYLE_DIM)?;
        w.write_str(s)?;
        self.write_ansi(w, ansi::STYLE_END)
    }

    fn write_bold(&self, w: &mut dyn Write, s: &str) -> fmt::Result {
        self.write_ansi(w, ansi::STYLE_BOLD)?;
        w.write_str(s)?;
        self.write_ansi(w, ansi::STYLE_END)
    }

    fn write_ansi(&self, w: &mut dyn Write, code: &str) -> fmt::Result {
        if self.color {
            w.write_str(code)
        } else {
            Ok(())
        }
    }

    fn configure_input_display<'b>(&self, display: InputDisplay<'b>) -> InputDisplay<'b> {
        let display = display.format(self.format);
        if self.color {
            display.color()
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
