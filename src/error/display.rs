use core::fmt::{self, Write};

use crate::error::ErrorDetails;
use crate::utils::WithFormatter;

const INPUT_PREFIX: &str = "> ";
const DEFAULT_MAX_WIDTH: usize = 80;

pub(crate) fn fmt_debug_error<'i, T>(error: T, f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    T: ErrorDetails<'i>,
{
    writeln!(
        f,
        "\n{:-<60}\n{}\n{:-<60}",
        "-- INPUT ERROR ",
        ErrorDisplay::from_formatter(error, f),
        "-",
    )
}

/// Provides configurable [`ErrorDetails`] formatting.
#[derive(Clone)]
pub struct ErrorDisplay<T> {
    error: T,
    max_width: Option<usize>,
}

impl<'i, T> ErrorDisplay<T>
where
    T: ErrorDetails<'i>,
{
    /// Create a new `ErrorDisplay` given an [`ErrorDetails`].
    pub fn new(error: T) -> Self {
        Self {
            error,
            max_width: Some(DEFAULT_MAX_WIDTH),
        }
    }

    /// Derive an `ErrorDisplay` from a [`fmt::Formatter`] with defaults.
    pub fn from_formatter(error: T, f: &fmt::Formatter<'_>) -> Self {
        let _ = f;
        Self::new(error)
    }

    /// Set the `max-width` for wrapping error output.
    pub fn max_width(mut self, value: Option<usize>) -> Self {
        self.max_width = value;
        self
    }

    /// Writes the [`ErrorDetails`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        let error = &self.error;
        let root_context = error.root_context();
        let input = error.input();
        writeln!(
            w,
            "error attempting to {}: {}",
            root_context.operation(),
            WithFormatter(|f| self.error.description(f)),
        )?;
        w.write_str(INPUT_PREFIX)?;
        if error.span().is_empty() {
            writeln!(w, "{}", input)?;
        } else if let Some((_before, _after)) = input.split_sub(error.span()) {
            // before.display().max(40)
            write!(w, "{}", input)?;
        } else {
            write!(w, "{}", input)?;
        }
        write!(w, "\ncontext bracktrace:\n{}", error.full_context())
    }
}

impl<'i, T> fmt::Debug for ErrorDisplay<T>
where
    T: ErrorDetails<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

impl<'i, T> fmt::Display for ErrorDisplay<T>
where
    T: ErrorDetails<'i>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}
