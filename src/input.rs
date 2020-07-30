use core::fmt::{self, Write};
use core::str;

use crate::error::{EndOfInput, Expected, Utf8Error};
use crate::reader::Reader;

/// Constructs a new `Input` from a byte slice.
#[inline(always)]
#[allow(unsafe_code)]
#[must_use = "input must be consumed"]
pub fn input(slice: &[u8]) -> &Input {
    // Cast the slice reference to a pointer.
    let slice_ptr: *const [u8] = slice;
    // Cast the slice pointer to a `Input` pointer.
    // The compiler allows this as the types are compatible.
    // This cast is safe as `Input` is a wrapper around [u8].
    // As with std::path::Path, `Input` is not marked
    // repr(transparent) or repr(C).
    let input_ptr = slice_ptr as *const Input;
    // Re-borrow the `Input` pointer as a `Input` reference.
    // This is safe as the lifetime from the slice is carried
    // from the slice reference to the `Input` reference.
    unsafe { &*input_ptr }
}

/// `Input` is an immutable wrapper around bytes to be processed.
///
/// It can only be created via [`dangerous::input`] as so to clearly
/// point out where user-generated / dangerous input is consumed.
///
/// It is used along with [`dangerous::Reader`] to process the input.
///
/// # Formatting
///
/// `Input` implements both [`fmt::Debug`] and [`fmt::Display`] with
/// support for pretty printing as shown below.
///
/// | [`fmt::Display`] | `"hello ♥"`                    | `&[0xFF, b'a', 0xFF]` |
/// | ---------------- | ------------------------------ | --------------------- |
/// | `"{}"`           | `<68 65 6c 6c 6f 20 e2 99 a5>` | `<ff 61 ff>`          |
/// | `"{:#}"`         | `"hello ♥"`                    | `<ff 'a' ff>`         |
/// | `"{:.2}"`        | `<68 65 ..>`                   | `<ff 61 ..>`          |
/// | `"{:#.2}"`       | `"he.." (truncated)`           | `<ff 'a' ..>`         |
#[derive(PartialEq, Eq)]
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

    #[inline(always)]
    pub const fn reader<E>(&self) -> Reader<'_, E> {
        Reader::new(self)
    }

    /// Returns the underlying byte slice.
    ///
    /// The naming of this function is to a degree hyperbole,
    /// and should not be necessarily taken as proof of something
    /// dangerous or memory unsafe. It is named this way simply
    /// for users to clearly note where the panic-free guarantees
    /// end when handling the input.
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
    /// Returns [`EndOfInput`] if the the input is empty.
    #[inline(always)]
    pub fn to_dangerous_non_empty(&self) -> Result<&[u8], EndOfInput<'_>> {
        if self.is_empty() {
            Ok(self.as_dangerous())
        } else {
            Err(EndOfInput {
                input: self,
                expected: Expected::LengthMin(1),
            })
        }
    }

    /// Parses the underlying byte slice as str slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`Utf8Error`] if the the input was not valid UTF-8.
    #[inline(always)]
    pub fn to_dangerous_str(&self) -> Result<&str, Utf8Error<'_>> {
        str::from_utf8(self.as_dangerous()).map_err(|error| Utf8Error { input: self, error })
    }

    /// Parses the underlying byte slice as str slice.
    ///
    /// See `as_dangerous` for naming.
    ///
    /// # Errors
    ///
    /// Returns [`EndOfInput`] if the the input is empty or [`Utf8Error`]
    /// if the the input was not valid UTF-8.
    #[inline(always)]
    pub fn to_dangerous_non_empty_str<'i, E>(&'i self) -> Result<&'i str, E>
    where
        E: From<Utf8Error<'i>>,
        E: From<EndOfInput<'i>>,
    {
        if self.is_empty() {
            Ok(self.to_dangerous_str()?)
        } else {
            Err(E::from(EndOfInput {
                input: self,
                expected: Expected::LengthMin(1),
            }))
        }
    }

    /// Returns the first byte in the input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn first(&self) -> Result<u8, EndOfInput<'_>> {
        self.as_dangerous()
            .first()
            .copied()
            .ok_or_else(|| EndOfInput {
                input: self,
                expected: Expected::Length(1),
            })
    }

    /// Splits the input into the first byte and whatever remains.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is empty.
    #[inline(always)]
    pub(crate) fn split_first(&self) -> Result<(u8, &Input), EndOfInput<'_>> {
        let (head, tail) = self.split_at(1)?;
        Ok((head.first()?, tail))
    }

    /// Splits the input into two at `mid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `mid > self.len()`.
    #[inline(always)]
    pub(crate) fn split_at(&self, mid: usize) -> Result<(&Input, &Input), EndOfInput<'_>> {
        if mid > self.len() {
            Err(EndOfInput {
                input: self,
                expected: Expected::Length(mid),
            })
        } else {
            let (head, tail) = self.as_dangerous().split_at(mid);
            Ok((input(head), input(tail)))
        }
    }

    /// Splits the input into two at `max`.
    #[inline(always)]
    pub(crate) fn split_max(&self, max: usize) -> (&Input, &Input) {
        if max > self.len() {
            (self, input(&[]))
        } else {
            let (head, tail) = self.as_dangerous().split_at(max);
            (input(head), input(tail))
        }
    }

    /// Splits the input when the provided function returns `false`.
    #[inline(always)]
    pub(crate) fn split_while<'i, F, E>(&'i self, f: F) -> Result<(&'i Input, &'i Input), E>
    where
        F: Fn(&'i Input, u8) -> Result<bool, E>,
    {
        let bytes = self.as_dangerous();
        for (i, byte) in bytes.iter().enumerate() {
            let (head, tail) = bytes.split_at(i);
            let tail = input(tail);
            if !f(&tail, *byte)? {
                return Ok((input(head), tail));
            }
        }
        Ok((self, input(&[])))
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
        f.debug_tuple("Input")
            .field(&format_args!("{}", &self))
            .finish()
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            if let Ok(s) = self.to_dangerous_str() {
                f.write_char('"')?;
                if let Some(max) = f.precision() {
                    for c in s.chars().take(max) {
                        f.write_char(c)?;
                    }
                    f.write_str("..\" (truncated)")
                } else {
                    f.write_str(s)?;
                    f.write_char('"')
                }
            } else {
                fmt_byte_str(f, self, |f, b| {
                    f.write_char('\'')?;
                    f.write_char(b as char)?;
                    f.write_char('\'')
                })
            }
        } else {
            fmt_byte_str(f, self, |f, b| write!(f, "{:x}", b))
        }
    }
}

fn fmt_byte_str<F>(f: &mut fmt::Formatter<'_>, input: &Input, ascii_graphic: F) -> fmt::Result
where
    F: Fn(&mut fmt::Formatter<'_>, u8) -> fmt::Result,
{
    let (input, has_more) = if let Some(max) = f.precision() {
        (input.split_max(max).0, max < input.len())
    } else {
        (input, false)
    };
    let mut byte_iter = input.as_dangerous().iter();
    let fmt_byte = |f: &mut fmt::Formatter<'_>, b: u8| {
        if b.is_ascii_graphic() {
            ascii_graphic(f, b)
        } else {
            write!(f, "{:x}", b)
        }
    };
    f.write_char('<')?;
    if let Some(byte) = byte_iter.next() {
        fmt_byte(f, *byte)?;
    }
    for byte in byte_iter {
        f.write_char(' ')?;
        fmt_byte(f, *byte)?;
    }
    if has_more {
        f.write_str(" ..")?;
    }
    f.write_char('>')
}
