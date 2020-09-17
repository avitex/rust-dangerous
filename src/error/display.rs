use core::fmt::{self, Write};

use crate::error::{Context, ErrorDetails};
use crate::utils::WithFormatter;

const INPUT_PREFIX: &str = "> ";
const DEFAULT_MAX_WIDTH: usize = 80;

/// Provides configurable [`ErrorDetails`] formatting.
#[derive(Clone)]
pub struct ErrorDisplay<'a, T> {
    error: &'a T,
    banner: bool,
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
            max_width: Some(DEFAULT_MAX_WIDTH),
        }
    }

    /// Derive an `ErrorDisplay` from a [`fmt::Formatter`] with defaults.
    pub fn from_formatter(error: &'a T, f: &fmt::Formatter<'_>) -> Self {
        let _ = f;
        Self::new(error)
    }

    /// Set whether or not a banner should printed around the error.
    pub fn banner(mut self, value: bool) -> Self {
        self.banner = value;
        self
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
        if self.banner {
            w.write_str("\n-- INPUT ERROR ---------------------------------------------\n")?;
            self.write_inner(w)?;
            w.write_str("\n------------------------------------------------------------\n")
        } else {
            self.write_inner(w)
        }
    }

    fn write_inner<W>(&self, w: &mut W) -> fmt::Result
    where
        W: Write,
    {
        let error = &self.error;
        let context_stack = error.context_stack();
        let input = error.input();
        writeln!(
            w,
            "error attempting to {}: {}",
            context_stack.root().operation(),
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
        write!(w, "\ncontext bracktrace:")?;
        let write_success = context_stack.walk(&mut |i, c| {
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
