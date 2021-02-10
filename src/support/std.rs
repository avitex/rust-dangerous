#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<'i> crate::error::External<'i> for std::net::AddrParseError {
    fn expected(&self) -> Option<&'static str> {
        Some("IP address")
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for crate::error::Invalid {}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for crate::error::Fatal {}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<'i, S> std::error::Error for crate::error::Expected<'i, S> where S: crate::error::Backtrace {}
