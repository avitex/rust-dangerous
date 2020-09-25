mod internal;

use core::{fmt, str};
use core::convert::Infallible;

use crate::display::InputDisplay;
use crate::error::{ExpectedContext, ExpectedLength, ExpectedValid, FromContext, OperationContext};
use crate::reader::Reader;
use crate::util::{is_sub_slice, utf8};

/// Creates a new `Input` from a byte slice.
///
/// It is recommended to use this directly from the crate as `dangerous::input()`,
/// not as an import via `use` as shown below, as you lose the discoverability.
///
/// ```
/// use dangerous::input; // bad
///
/// dangerous::input(b"hello"); // do this instead
/// ```
#[inline(always)]
#[allow(unsafe_code)]
#[must_use = "input must be consumed"]
pub fn input(slice: &[u8]) -> &Input {
    // Cast the slice reference to a pointer.
    let slice_ptr: *const [u8] = slice;
    // Cast the slice pointer to a `Input` pointer.
    //
    // The compiler allows this as the types are compatible. This cast is safe
    // as `Input` is a wrapper around [u8]. As with std::path::Path, `Input` is
    // not marked repr(transparent) or repr(C).
    let input_ptr = slice_ptr as *const Input;
    // Re-borrow the `Input` pointer as a `Input` reference.
    //
    // This is safe as the lifetime from the slice is carried from the slice
    // reference to the `Input` reference.
    unsafe { &*input_ptr }
}

/// `Input` is an immutable wrapper around bytes to be processed.
///
/// It can only be created via [`dangerous::input()`] as so to clearly point out
/// where untrusted / dangerous input is consumed.
///
/// It is used along with [`Reader`] to process the input.
///
/// # Formatting
///
/// `Input` implements both [`fmt::Debug`] and [`fmt::Display`] with support for
/// pretty printing. See [`InputDisplay`] for formatting options.
///
/// [`dangerous::input()`]: crate::input()
#[derive(Eq, PartialEq)]
#[must_use = "input must be consumed"]
pub struct Input([u8]);

impl Input {
    /// Returns the underlying byte slice length.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.as_dangerous().len()
    }

    /// Returns `true` if the underlying byte slice length is zero.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.as_dangerous().is_empty()
    }

    /// Returns `true` if the underlying byte slice for `parent` contains that
    /// of `self` in the same section of memory with no bounds out of range.
    pub fn is_within(&self, parent: &Input) -> bool {
        is_sub_slice(parent.as_dangerous(), self.as_dangerous())
    }

    /// Returns an [`InputDisplay`] for formatting.
    #[inline(always)]
    pub const fn display(&self) -> InputDisplay<'_> {
        InputDisplay::new(self)
    }

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole, and should not be
    /// necessarily taken as proof of something dangerous or memory unsafe. It
    /// is named this way simply for users to clearly note where the panic-free
    /// guarantees end when handling the input.
    #[inline(always)]
    pub const fn as_dangerous(&self) -> &[u8] {
        &self.0
    }

    /// Returns the underlying byte slice if it is not empty.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedLength`] if the the input is empty.
    #[inline]
    pub fn to_dangerous_non_empty<'i, E>(&'i self) -> Result<&'i [u8], E>
    where
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                input: self,
                context: ExpectedContext {
                    operation: "convert input to non-empty slice",
                    expected: "non-empty input",
                },
            }))
        } else {
            Ok(self.as_dangerous())
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the the input could never be valid UTF-8
    /// and [`ExpectedLength`] if a UTF-8 code point was cut short. This is
    /// useful when parsing potentially incomplete buffers.
    #[inline]
    pub fn to_dangerous_str<'i, E>(&'i self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        match str::from_utf8(self.as_dangerous()) {
            Ok(s) => Ok(s),
            Err(utf8_err) => match utf8_err.error_len() {
                None => {
                    let invalid = &self.as_dangerous()[utf8_err.valid_up_to()..];
                    Err(E::from(ExpectedLength {
                        min: utf8::char_len(invalid[0]),
                        max: None,
                        span: input(invalid),
                        input: self,
                        context: ExpectedContext {
                            operation: "convert input to str",
                            expected: "complete utf-8 code point",
                        },
                    }))
                }
                Some(error_len) => {
                    let bytes = self.as_dangerous();
                    let error_start = utf8_err.valid_up_to();
                    let error_end = error_start + error_len;
                    Err(E::from(ExpectedValid {
                        span: input(&bytes[error_start..error_end]),
                        input: self,
                        context: ExpectedContext {
                            operation: "convert input to str",
                            expected: "utf-8 code point",
                        },
                        retry_requirement: None,
                    }))
                }
            },
        }
    }

    /// Decodes the underlying byte slice into a UTF-8 `str` slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`ExpectedValid`] if the the input could never be valid UTF-8 and
    /// [`ExpectedLength`] if a UTF-8 code point was cut short or the input is
    /// empty. This is useful when parsing potentially incomplete buffers.
    #[inline]
    pub fn to_dangerous_non_empty_str<'i, E>(&'i self) -> Result<&'i str, E>
    where
        E: From<ExpectedValid<'i>>,
        E: From<ExpectedLength<'i>>,
    {
        if self.is_empty() {
            Err(E::from(ExpectedLength {
                min: 1,
                max: None,
                span: self,
                input: self,
                context: ExpectedContext {
                    operation: "convert input to non-empty str",
                    expected: "non empty input",
                },
            }))
        } else {
            self.to_dangerous_str()
        }
    }

    /// Create a reader with the expectation all of the input is read.
    ///
    /// # Errors
    ///
    /// Returns an error if either the provided function does, or there is
    /// trailing input.
    pub fn read_all<'i, F, O, E>(&'i self, f: F) -> Result<O, E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<O, E>,
        E: FromContext<'i>,
        E: From<ExpectedLength<'i>>,
    {
        let mut r = Reader::new(self);
        let ok = r.context(OperationContext("read all"), f)?;
        if r.at_end() {
            Ok(ok)
        } else {
            Err(E::from(ExpectedLength {
                min: 0,
                max: Some(0),
                span: r.take_remaining(),
                input: self,
                context: ExpectedContext {
                    operation: "read all",
                    expected: "no trailing input",
                },
            }))
        }
    }

    /// Create a reader to read a part of the input and return the rest.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided function does.
    pub fn read_partial<'i, F, O, E>(&'i self, f: F) -> Result<(O, &'i Input), E>
    where
        F: FnOnce(&mut Reader<'i, E>) -> Result<O, E>,
        E: FromContext<'i>,
    {
        let mut r = Reader::new(self);
        let ok = r.context(OperationContext("read partial"), f)?;
        Ok((ok, r.take_remaining()))
    }

    /// Create a reader to read a part of the input and return the rest
    /// without any errors.
    pub fn read_infallible<'i, F, O>(&'i self, f: F) -> (O, &'i Input)
    where
        F: FnOnce(&mut Reader<'i, Infallible>) -> O,
    {
        let mut r = Reader::new(self);
        let ok = f(&mut r);
        (ok, r.take_remaining())
    }
}

impl AsRef<Input> for Input {
    fn as_ref(&self) -> &Input {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Equality

impl PartialEq<[u8]> for Input {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl PartialEq<[u8]> for &Input {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_dangerous() == other
    }
}

impl PartialEq<Input> for [u8] {
    fn eq(&self, other: &Input) -> bool {
        other.as_dangerous() == self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Formatting

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = InputDisplay::from_formatter(self, f);
        f.debug_tuple("Input").field(&display).finish()
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InputDisplay::from_formatter(self, f).fmt(f)
    }
}
