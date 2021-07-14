use crate::input::{Prefix, String};

unsafe impl<'i> Prefix<String<'i>> for char {
    #[inline(always)]
    fn is_prefix_of(self, input: &String<'i>) -> bool {
        match input.as_dangerous().chars().next() {
            Some(c) => c == self,
            None => false,
        }
    }
}

unsafe impl<'i> Prefix<String<'i>> for &str {
    #[inline(always)]
    fn is_prefix_of(self, input: &String<'i>) -> bool {
        input.as_dangerous().starts_with(self)
    }
}
