use crate::input::{Bytes, Prefix};
use crate::util::utf8::CharBytes;

unsafe impl<'i> Prefix<Bytes<'i>> for u8 {
    #[inline(always)]
    fn is_prefix_of(self, input: &Bytes<'i>) -> bool {
        input.as_dangerous().starts_with(&[self])
    }
}

unsafe impl<'i> Prefix<Bytes<'i>> for char {
    #[inline(always)]
    fn is_prefix_of(self, input: &Bytes<'i>) -> bool {
        let bytes = CharBytes::from(self);
        input.as_dangerous().starts_with(bytes.as_bytes())
    }
}

unsafe impl<'i> Prefix<Bytes<'i>> for &[u8] {
    #[inline(always)]
    fn is_prefix_of(self, input: &Bytes<'i>) -> bool {
        input.as_dangerous().starts_with(self)
    }
}

unsafe impl<'i> Prefix<Bytes<'i>> for &str {
    #[inline(always)]
    fn is_prefix_of(self, input: &Bytes<'i>) -> bool {
        input.as_dangerous().starts_with(self.as_bytes())
    }
}

unsafe impl<'i, const N: usize> Prefix<Bytes<'i>> for &[u8; N] {
    #[inline(always)]
    fn is_prefix_of(self, input: &Bytes<'i>) -> bool {
        input.as_dangerous().starts_with(&self[..])
    }
}
