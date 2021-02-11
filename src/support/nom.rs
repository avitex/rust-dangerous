use core::any::Any;

use nom::error::{Error, ErrorKind};
#[cfg(feature = "alloc")]
use nom::error::{VerboseError, VerboseErrorKind};
use nom::{Err, Needed};

#[cfg(feature = "retry")]
use crate::error::RetryRequirement;
use crate::error::{Context, External, Operation, WithContext};
use crate::fmt;
use crate::input::Span;

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
    fn span(&self) -> Option<Span> {
        match self {
            Err::Error(err) | Err::Failure(err) => err.span(),
            Err::Incomplete(_) => None,
        }
    }

    #[cfg(feature = "retry")]
    fn retry_requirement(&self) -> Option<RetryRequirement> {
        match self {
            Err::Error(_) | Err::Failure(_) => None,
            Err::Incomplete(Needed::Unknown) => RetryRequirement::new(1),
            Err::Incomplete(Needed::Size(s)) => RetryRequirement::new(s.get()),
        }
    }

    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        match self {
            Err::Error(err) | Err::Failure(err) => err.push_backtrace(error),
            Err::Incomplete(_) => error,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Basic

struct NomContext {
    kind: ErrorKind,
    span: Span,
}

impl Context for NomContext {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }

    fn operation(&self) -> &dyn Operation {
        &self.kind
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nom")))]
impl Operation for ErrorKind {
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        w.write_str(self.description())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nom")))]
impl<'i, I> External<'i> for Error<I>
where
    I: AsBytes<'i>,
{
    fn span(&self) -> Option<Span> {
        Some(self.input.as_bytes().into())
    }

    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error.with_context(NomContext {
            span: self.input.as_bytes().into(),
            kind: self.code,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Verbose

#[cfg(feature = "alloc")]
struct NomVerboseContext {
    kind: VerboseErrorKind,
    span: Span,
}

#[cfg(feature = "alloc")]
impl Context for NomVerboseContext {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }

    fn operation(&self) -> &dyn Operation {
        &self.kind
    }

    fn has_expected(&self) -> bool {
        match self.kind {
            VerboseErrorKind::Char(_) | VerboseErrorKind::Context(_) => true,
            VerboseErrorKind::Nom(_) => false,
        }
    }

    fn expected(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match self.kind {
            VerboseErrorKind::Char(c) => {
                w.write_str("character '")?;
                w.write_char(c)?;
                w.write_char('\'')
            }
            VerboseErrorKind::Context(c) => w.write_str(c),
            VerboseErrorKind::Nom(_) => Err(fmt::Error),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "nom", feature = "alloc"))))]
impl Operation for VerboseErrorKind {
    fn description(&self, w: &mut dyn fmt::Write) -> fmt::Result {
        match self {
            VerboseErrorKind::Context(_) => w.write_str("<context>"),
            VerboseErrorKind::Char(_) => w.write_str("consume input"),
            VerboseErrorKind::Nom(kind) => Operation::description(kind, w),
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
    fn span(&self) -> Option<Span> {
        self.errors.get(0).map(|(input, _)| input.as_bytes().into())
    }

    fn push_backtrace<E>(self, mut error: E) -> E
    where
        E: WithContext<'i>,
    {
        for (input, kind) in self.errors.into_iter() {
            error = error.with_context(NomVerboseContext {
                span: input.as_bytes().into(),
                kind,
            });
        }
        error
    }
}
