use core::fmt::{Debug, Display, Formatter};

pub(crate) use core::fmt::{Error, Result, Write};

pub trait FormatterBase {
    fn as_dyn_mut(&mut self) -> &mut dyn FormatterBase;

    fn alternate(&self) -> bool;
    fn precision(&self) -> Option<usize>;

    fn write_str(&mut self, s: &str) -> Result;
    fn write_char(&mut self, c: char) -> Result;
    fn write_usize(&mut self, v: usize) -> Result;
    fn write_display(&mut self, v: &dyn DisplayBase) -> Result;

    fn debug_str(&mut self, s: &str) -> Result;
    fn debug_tuple(&mut self, name: &str, fields: &[&dyn DebugBase]) -> Result;
    fn debug_struct(&mut self, name: &str, fields: &[(&'static str, &dyn DebugBase)]) -> Result;
}

impl<'a> FormatterBase for Formatter<'a> {
    fn as_dyn_mut(&mut self) -> &mut dyn FormatterBase {
        self
    }

    fn alternate(&self) -> bool {
        Formatter::alternate(self)
    }

    fn precision(&self) -> Option<usize> {
        Formatter::precision(self)
    }

    fn write_str(&mut self, s: &str) -> Result {
        <Formatter<'_> as Write>::write_str(self, s)
    }

    fn write_char(&mut self, c: char) -> Result {
        <Formatter<'_> as Write>::write_char(self, c)
    }

    fn write_usize(&mut self, v: usize) -> Result {
        Display::fmt(&v, self)
    }

    fn write_display(&mut self, v: &dyn DisplayBase) -> Result {
        DisplayBase::fmt(v, self)
    }

    fn debug_str(&mut self, s: &str) -> Result {
        Debug::fmt(s, self)
    }

    fn debug_tuple(&mut self, name: &str, fields: &[&dyn DebugBase]) -> Result {
        let mut debug = Formatter::debug_tuple(self, name);

        struct Adapter<'a>(&'a dyn DebugBase);

        impl<'a> Debug for Adapter<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                DebugBase::fmt(self.0, f)
            }
        }

        for value in fields {
            let _ = debug.field(&Adapter(*value));
        }

        debug.finish()
    }

    fn debug_struct(&mut self, name: &str, fields: &[(&'static str, &dyn DebugBase)]) -> Result {
        let mut debug = Formatter::debug_struct(self, name);

        struct Adapter<'a>(&'a dyn DebugBase);

        impl<'a> Debug for Adapter<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                DebugBase::fmt(self.0, f)
            }
        }

        for (name, value) in fields {
            let _ = debug.field(name, &Adapter(*value));
        }

        debug.finish()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait DebugBase {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result;
}

impl DebugBase for usize {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        f.write_usize(*self)
    }
}

impl DebugBase for str {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        f.debug_str(self)
    }
}

impl<T> DebugBase for Option<T>
where
    T: DebugBase,
{
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        match self {
            None => f.debug_tuple("None", &[]),
            Some(value) => f.debug_tuple("Some", &[value]),
        }
    }
}

impl DebugBase for core::num::NonZeroUsize {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        f.debug_tuple("NonZeroUsize", &[&self.get()])
    }
}

impl<T> DebugBase for &T
where
    T: DebugBase + ?Sized,
{
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        (**self).fmt(f)
    }
}

///////////////////////////////////////////////////////////////////////////////

///
pub trait DisplayBase {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result;
}

impl<T> DisplayBase for &T
where
    T: DisplayBase + ?Sized,
{
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        (**self).fmt(f)
    }
}

impl DisplayBase for str {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        f.write_str(self)
    }
}

impl DisplayBase for usize {
    fn fmt(&self, f: &mut dyn FormatterBase) -> Result {
        f.write_usize(*self)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub(crate) struct Writer<'a, T: ?Sized>(&'a mut T);

impl<'a, T: ?Sized> Writer<'a, T>
where
    T: FormatterBase,
{
    pub fn new(f: &'a mut T) -> Self {
        Self(f)
    }
}

impl<'a, T: ?Sized> Write for Writer<'a, T>
where
    T: FormatterBase,
{
    fn write_str(&mut self, s: &str) -> Result {
        self.0.write_str(s)
    }

    fn write_char(&mut self, c: char) -> Result {
        self.0.write_char(c)
    }
}
