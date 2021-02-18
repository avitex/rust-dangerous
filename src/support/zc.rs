#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::RootBacktrace {
    type Static = crate::error::RootBacktrace;
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "zc", feature = "alloc"))))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::FullBacktrace {
    type Static = crate::error::FullBacktrace;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::CoreContext {
    type Static = crate::error::CoreContext;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::Fatal {
    type Static = crate::error::Fatal;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::Invalid {
    type Static = crate::error::Invalid;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::RetryRequirement {
    type Static = crate::error::RetryRequirement;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o, S> zc::Dependant<'o> for crate::error::Expected<'o, S>
where
    S: zc::Dependant<'o>,
{
    type Static = crate::error::Expected<'static, S::Static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::ExpectedLength<'o> {
    type Static = crate::error::ExpectedLength<'static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::ExpectedValid<'o> {
    type Static = crate::error::ExpectedValid<'static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::error::ExpectedValue<'o> {
    type Static = crate::error::ExpectedValue<'static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::input::Span {
    type Static = crate::input::Span;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::input::Bytes<'o> {
    type Static = crate::input::Bytes<'static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::input::String<'o> {
    type Static = crate::input::String<'static>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'o> zc::Dependant<'o> for crate::input::MaybeString<'o> {
    type Static = crate::input::MaybeString<'static>;
}
