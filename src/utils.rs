use core::fmt;

pub(crate) struct ByteCount(pub(crate) usize);

impl fmt::Display for ByteCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => f.write_str("no bytes"),
            1 => f.write_str("1 byte"),
            n => write!(f, "{} bytes", n),
        }
    }
}

pub(crate) struct WithFormatter<T>(pub(crate) T)
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<T> fmt::Display for WithFormatter<T>
where
    T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}
