use crate::error::{ExternalContext, WithContext};

impl<'i> crate::error::External<'i> for () {}

impl<'i> crate::error::External<'i> for core::num::ParseFloatError {
    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error.with_context(ExternalContext {
            operation: Some("parse from string"),
            expected: Some("float"),
        })
    }
}

impl<'i> crate::error::External<'i> for core::num::ParseIntError {
    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error.with_context(ExternalContext {
            operation: Some("parse from string"),
            expected: Some("integer"),
        })
    }
}

impl<'i> crate::error::External<'i> for core::str::ParseBoolError {
    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error.with_context(ExternalContext {
            operation: Some("parse from string"),
            expected: Some("boolean"),
        })
    }
}

impl<'i> crate::error::External<'i> for core::char::ParseCharError {
    fn push_backtrace<E>(self, error: E) -> E
    where
        E: WithContext<'i>,
    {
        error.with_context(ExternalContext {
            operation: Some("parse from string"),
            expected: Some("char"),
        })
    }
}
