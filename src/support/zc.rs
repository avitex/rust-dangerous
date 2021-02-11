#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl zc::NoInteriorMut for crate::error::RootBacktrace {}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "zc", feature = "alloc"))))]
unsafe impl zc::NoInteriorMut for crate::error::FullBacktrace {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl zc::NoInteriorMut for crate::error::CoreContext {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl zc::NoInteriorMut for crate::error::Fatal {}

#[cfg(feature = "retry")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "zc", feature = "retry"))))]
unsafe impl zc::NoInteriorMut for crate::error::Invalid {}

#[cfg(feature = "retry")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "zc", feature = "retry"))))]
unsafe impl zc::NoInteriorMut for crate::error::RetryRequirement {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::error::ExpectedLength<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i, S> zc::NoInteriorMut for crate::error::Expected<'i, S> where S: zc::NoInteriorMut {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::error::ExpectedValid<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::error::ExpectedValue<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::input::Bytes<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::input::String<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i> zc::NoInteriorMut for crate::input::MaybeString<'i> {}

#[cfg_attr(docsrs, doc(cfg(feature = "zc")))]
unsafe impl<'i, E, I> zc::NoInteriorMut for crate::reader::Reader<'i, E, I>
where
    E: zc::NoInteriorMut,
    I: zc::NoInteriorMut + crate::input::Input<'i>,
{
}
