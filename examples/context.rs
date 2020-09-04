use dangerous::{AdditionalContext, Expected, ExpectedLength, ExpectedValid, ExpectedValue, Input};

struct Error<'i> {
    expected: Expected<'i>,
    additional: Option<AdditionalContext<'i>>,
}

impl<'i> dangerous::FromError<ExpectedValue<'i>> for Error<'i> {
    fn from_err(err: ExpectedValue<'i>) -> Self {
        Self {
            expected: err.into(),
            additional: None,
        }
    }

    fn from_err_ctx<C>(err: ExpectedValue<'i>, ctx: C) -> Self {
        Self {
            expected: err.into(),
            additional: None,
        }
    }
}

fn main() {}
