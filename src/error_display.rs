use core::fmt::{self, Write};

use crate::error::Error;

const INPUT_PREFIX: &str = "> ";
const DEFAULT_MAX_WIDTH: usize = 80;

/// Provides configurable [`Error`] formatting.
#[derive(Clone)]
pub struct ErrorDisplay<'a, T: ?Sized> {
    error: &'a T,
    max_width: Option<usize>,
}

impl<'a, T> ErrorDisplay<'a, T>
where
    T: ?Sized + Error,
{
    /// Create a new `ErrorDisplay` given an [`Error`].
    pub fn new(error: &'a T) -> Self {
        Self {
            error,
            max_width: Some(DEFAULT_MAX_WIDTH),
        }
    }

    /// Derive an `ErrorDisplay` from a [`fmt::Formatter`] with defaults.
    pub fn from_formatter(error: &'a T, f: &fmt::Formatter<'_>) -> Self {
        let _ = f;
        Self::new(error)
    }

    /// Set the `max-width` for wrapping error output.
    pub fn max_width(mut self, value: Option<usize>) -> Self {
        self.max_width = value;
        self
    }

    /// Writes the [`Error`] to a writer with the choosen format.
    ///
    /// # Errors
    ///
    /// Returns [`core::fmt::Error`] if failed to write.
    pub fn write<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        let error = self.error;
        let context = self.error.context();
        let input = context.input();
        writeln!(
            w,
            "expected {} while attempting to {}, instead found {}",
            WithFormatter(|f| self.error.expected_description(f)),
            context.operation(),
            WithFormatter(|f| self.error.found_description(f)),
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
        Ok(())
    }
}

impl<'a, T> fmt::Debug for ErrorDisplay<'a, T>
where
    T: ?Sized + Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

impl<'a, T> fmt::Display for ErrorDisplay<'a, T>
where
    T: ?Sized + Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct WithFormatter<T>(T)
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<T> fmt::Display for WithFormatter<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}
