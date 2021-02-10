use core::any::Any;

use nom::error::{Error, ErrorKind};
#[cfg(feature = "alloc")]
use nom::error::{VerboseError, VerboseErrorKind};
use nom::{Err, Needed};

#[cfg(feature = "retry")]
use crate::error::RetryRequirement;
use crate::error::{Context, External, WithContext};
use crate::fmt;

pub trait AsBytes<'i> {
    fn as_bytes(&self) -> &'i [u8];
}

impl<'i> AsBytes<'i> for &'i [u8] {
    fn as_bytes(&self) -> &'i [u8] {
        self
    }
}

impl<'i> AsBytes<'i> for &'i str {
    fn as_bytes(&self) -> &'i [u8] {
        str::as_bytes(self)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nom")))]
impl<'i, Ex> External<'i> for Err<Ex>
where
    Ex: External<'i>,
{
    fn span(&self) -> Option<&'i [u8]> {
        match self {
            Err::Error(err) | Err::Failure(err) => err.span(),
            Err::Incomplete(_) => None,
        }
    }

    fn operation(&self) -> Option<&'static str> {
        match self {
            Err::Error(err) | Err::Failure(err) => err.operation(),
            Err::Incomplete(_) => None,
        }
    }

    fn expected(&self) -> Option<&'static str> {
        match self {
            Err::Error(err) | Err::Failure(err) => err.expected(),
            Err::Incomplete(_) => None,
        }
    }

    #[cfg(feature = "retry")]
    fn retry_requirement(&self) -> Option<RetryRequirement> {
        match self {
            Err::Error(_) | Err::Incomplete(Needed::Unknown) => RetryRequirement::new(1),
            Err::Failure(_) => None,
            Err::Incomplete(Needed::Size(s)) => RetryRequirement::new(s.get()),
        }
    }

    fn push_child_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        match self {
            Err::Error(err) | Err::Failure(err) => err.push_child_backtrace(error),
            Err::Incomplete(_) => error,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nom")))]
impl<'i, I> External<'i> for Error<I>
where
    I: AsBytes<'i>,
{
    fn span(&self) -> Option<&'i [u8]> {
        Some(self.input.as_bytes())
    }

    fn operation(&self) -> Option<&'static str> {
        Some(error_kind_operation(self.code))
    }

    fn push_child_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nom")))]
impl Context for ErrorKind {
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self.description())
    }

    fn has_expected(&self) -> bool {
        false
    }

    fn expected(&self, _: &mut dyn fmt::Write) -> fmt::Result {
        Err(fmt::Error)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "nom", feature = "alloc"))))]
impl Context for VerboseErrorKind {
    fn operation(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match *self {
            VerboseErrorKind::Context(_) => w.write_str("<nom>"),
            VerboseErrorKind::Char(_) => w.write_str("<nom>"),
            VerboseErrorKind::Nom(kind) => w.write_str(error_kind_operation(kind)),
        }
    }

    fn has_expected(&self) -> bool {
        match self {
            VerboseErrorKind::Char(_) | VerboseErrorKind::Context(_) => true,
            VerboseErrorKind::Nom(_) => false,
        }
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match *self {
            VerboseErrorKind::Char(c) => {
                w.write_str("char '")?;
                w.write_char(c)?;
                w.write_char('\'')
            }
            VerboseErrorKind::Context(c) => w.write_str(c),
            VerboseErrorKind::Nom(_) => Err(fmt::Error),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "nom", feature = "alloc"))))]
impl<'i, I> External<'i> for VerboseError<I>
where
    I: AsBytes<'i>,
{
    fn span(&self) -> Option<&'i [u8]> {
        self.errors.get(0).map(|(input, _)| input.as_bytes())
    }

    fn operation(&self) -> Option<&'static str> {
        self.errors.get(0).map(|(_, kind)| match *kind {
            VerboseErrorKind::Context(_) => "<nom>",
            VerboseErrorKind::Char(_) => "<nom>",
            VerboseErrorKind::Nom(kind) => error_kind_operation(kind),
        })
    }

    fn push_child_backtrace<E>(self, mut error: E) -> E
    where
        E: WithContext<'i>,
    {
        for (_, code) in self.errors.into_iter().skip(1) {
            error = error.with_child_context(code);
        }
        error
    }
}

// Taken from: https://docs.rs/nom/6.1.0/src/nom/error.rs.html#487-543
//
// FIXME: remove this if `ErrorKind::description()` is changed to return a
// string with a `'static` lifetime.
#[rustfmt::skip]
fn error_kind_operation(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Tag                       => "Tag",
        ErrorKind::MapRes                    => "Map on Result",
        ErrorKind::MapOpt                    => "Map on Option",
        ErrorKind::Alt                       => "Alternative",
        ErrorKind::IsNot                     => "IsNot",
        ErrorKind::IsA                       => "IsA",
        ErrorKind::SeparatedList             => "Separated list",
        ErrorKind::SeparatedNonEmptyList     => "Separated non empty list",
        ErrorKind::Many0                     => "Many0",
        ErrorKind::Many1                     => "Many1",
        ErrorKind::Count                     => "Count",
        ErrorKind::TakeUntil                 => "Take until",
        ErrorKind::LengthValue               => "Length followed by value",
        ErrorKind::TagClosure                => "Tag closure",
        ErrorKind::Alpha                     => "Alphabetic",
        ErrorKind::Digit                     => "Digit",
        ErrorKind::AlphaNumeric              => "AlphaNumeric",
        ErrorKind::Space                     => "Space",
        ErrorKind::MultiSpace                => "Multiple spaces",
        ErrorKind::LengthValueFn             => "LengthValueFn",
        ErrorKind::Eof                       => "End of file",
        ErrorKind::Switch                    => "Switch",
        ErrorKind::TagBits                   => "Tag on bitstream",
        ErrorKind::OneOf                     => "OneOf",
        ErrorKind::NoneOf                    => "NoneOf",
        ErrorKind::Char                      => "Char",
        ErrorKind::CrLf                      => "CrLf",
        ErrorKind::RegexpMatch               => "RegexpMatch",
        ErrorKind::RegexpMatches             => "RegexpMatches",
        ErrorKind::RegexpFind                => "RegexpFind",
        ErrorKind::RegexpCapture             => "RegexpCapture",
        ErrorKind::RegexpCaptures            => "RegexpCaptures",
        ErrorKind::TakeWhile1                => "TakeWhile1",
        ErrorKind::Complete                  => "Complete",
        ErrorKind::Fix                       => "Fix",
        ErrorKind::Escaped                   => "Escaped",
        ErrorKind::EscapedTransform          => "EscapedTransform",
        ErrorKind::NonEmpty                  => "NonEmpty",
        ErrorKind::ManyMN                    => "Many(m, n)",
        ErrorKind::HexDigit                  => "Hexadecimal Digit",
        ErrorKind::OctDigit                  => "Octal digit",
        ErrorKind::Not                       => "Negation",
        ErrorKind::Permutation               => "Permutation",
        ErrorKind::ManyTill                  => "ManyTill",
        ErrorKind::Verify                    => "predicate verification",
        ErrorKind::TakeTill1                 => "TakeTill1",
        ErrorKind::TakeWhileMN               => "TakeWhileMN",
        ErrorKind::ParseTo                   => "Parse string to the specified type",
        ErrorKind::TooLarge                  => "Needed data size is too large",
        ErrorKind::Many0Count                => "Count occurrence of >=0 patterns",
        ErrorKind::Many1Count                => "Count occurrence of >=1 patterns",
        ErrorKind::Float                     => "Float",
        ErrorKind::Satisfy                   => "Satisfy",
    }
}
