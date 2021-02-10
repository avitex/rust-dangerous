impl<'i> crate::error::External<'i> for core::num::ParseFloatError {
    fn operation(&self) -> Option<&'static str> {
        Some("from str")
    }

    fn expected(&self) -> Option<&'static str> {
        Some("float")
    }
}

impl<'i> crate::error::External<'i> for core::num::ParseIntError {
    fn operation(&self) -> Option<&'static str> {
        Some("from str")
    }

    fn expected(&self) -> Option<&'static str> {
        Some("integer")
    }
}

impl<'i> crate::error::External<'i> for core::str::ParseBoolError {
    fn operation(&self) -> Option<&'static str> {
        Some("from str")
    }

    fn expected(&self) -> Option<&'static str> {
        Some("bool")
    }
}

impl<'i> crate::error::External<'i> for core::char::ParseCharError {
    fn operation(&self) -> Option<&'static str> {
        Some("from str")
    }

    fn expected(&self) -> Option<&'static str> {
        Some("char")
    }
}
