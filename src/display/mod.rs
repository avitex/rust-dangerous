//! Display support.

mod error;
mod input;
mod section;
mod section_unit;
mod unit;
mod writer;

pub(crate) mod fmt;

pub use self::error::ErrorDisplay;
pub use self::fmt::{DebugBase, DisplayBase, FormatterBase};
pub use self::input::{InputDisplay, PreferredFormat};

pub(crate) struct ByteCount(pub(crate) usize);

impl DisplayBase for ByteCount {
    fn fmt(&self, f: &mut dyn FormatterBase) -> fmt::Result {
        match self.0 {
            0 => f.write_str("no bytes"),
            1 => f.write_str("1 byte"),
            n => {
                f.write_usize(n)?;
                f.write_str(" bytes")
            }
        }
    }
}

forward_fmt!(impl Display for ByteCount);
