//! All errors supported.

use core::{fmt, str};

use crate::input::Input;

/// `InputError` contains no information about what happened, 
/// only that an error occurred.
#[derive(Debug, PartialEq)]
pub struct InputError;

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("invalid input")
    }
}

///////////////////////////////////////////////////////////////////////////////
// UTF-8 error

#[derive(Debug, PartialEq)]
pub struct Utf8Error<'i> {
    pub(crate) input: &'i Input,
    pub(crate) error: str::Utf8Error,
}

// impl<'i> fmt::Display for Utf8Error<'i> {
//     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//         unimplemented!()
//     }
// }

impl<'i> From<Utf8Error<'i>> for InputError {
    fn from(_: Utf8Error<'i>) -> Self {
        InputError
    }
}

///////////////////////////////////////////////////////////////////////////////
// End of input error

#[derive(Debug, PartialEq)]
pub struct EndOfInput<'i> {
    pub(crate) input: &'i Input,
    pub(crate) expected: Expected<'i>,
}

// impl<'i> fmt::Display for EndOfInput<'i> {
//     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//         unimplemented!()
//     }
// }

impl<'i> From<EndOfInput<'i>> for InputError {
    #[inline(always)]
    fn from(_: EndOfInput<'i>) -> Self {
        InputError
    }
}

///////////////////////////////////////////////////////////////////////////////
// Unexpected input error

#[derive(Debug, PartialEq)]
pub struct UnexpectedInput<'i> {
    pub(crate) input: &'i Input,
    pub(crate) expected: Expected<'i>,
}

// impl<'i> fmt::Display for UnexpectedInput<'i> {
//     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//         unimplemented!()
//     }
// }

impl<'i> From<UnexpectedInput<'i>> for InputError {
    #[inline(always)]
    fn from(_: UnexpectedInput<'i>) -> Self {
        InputError
    }
}

///////////////////////////////////////////////////////////////////////////////
// Trailing input error

#[derive(Debug, PartialEq)]
pub struct TrailingInput<'i> {
    pub(crate) before: &'i Input,
    pub(crate) trailing: &'i Input,
}

// impl<'i> fmt::Display for TrailingInput<'i> {
//     fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
//         unimplemented!()
//     }
// }

impl<'i> From<TrailingInput<'i>> for InputError {
    #[inline(always)]
    fn from(_: TrailingInput<'i>) -> Self {
        InputError
    }
}

///////////////////////////////////////////////////////////////////////////////
// Error support

#[derive(Debug, PartialEq)]
pub enum Expected<'a> {
    Bytes(&'a [u8]),
    Length(usize),
    LengthMin(usize),
    Description(&'static str),
}
