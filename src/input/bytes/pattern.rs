use crate::input::{Bytes, Pattern};
use crate::util::fast;

///////////////////////////////////////////////////////////////////////////////
// Fn pattern

unsafe impl<'i, F> Pattern<Bytes<'i>> for F
where
    F: FnMut(u8) -> bool,
{
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .position(self)
            .map(|i| (i, 1))
    }

    fn find_reject(mut self, input: &Bytes<'i>) -> Option<usize> {
        input
            .as_dangerous()
            .iter()
            .copied()
            .enumerate()
            .find_map(|(i, b)| if (self)(b) { None } else { Some(i) })
    }
}

///////////////////////////////////////////////////////////////////////////////
// Token pattern

unsafe impl<'i> Pattern<Bytes<'i>> for u8 {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_u8_match(self, input.as_dangerous()).map(|index| (index, 1))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_u8_reject(self, input.as_dangerous())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Sub-slice pattern

unsafe impl<'i> Pattern<Bytes<'i>> for &[u8] {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self, input.as_dangerous()).map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i, const N: usize> Pattern<Bytes<'i>> for &[u8; N] {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self, input.as_dangerous()).map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self, input.as_dangerous())
    }
}

unsafe impl<'i> Pattern<Bytes<'i>> for &str {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        fast::find_slice_match(self.as_bytes(), input.as_dangerous())
            .map(|index| (index, self.len()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        fast::find_slice_reject(self.as_bytes(), input.as_dangerous())
    }
}

///////////////////////////////////////////////////////////////////////////////
// Regex pattern

#[cfg(feature = "regex")]
unsafe impl<'i> Pattern<Bytes<'i>> for &regex::bytes::Regex {
    fn find_match(self, input: &Bytes<'i>) -> Option<(usize, usize)> {
        regex::bytes::Regex::find(self, input.as_dangerous())
            .map(|m| (m.start(), m.end() - m.start()))
    }

    fn find_reject(self, input: &Bytes<'i>) -> Option<usize> {
        let mut maybe_reject = 0;
        loop {
            match regex::bytes::Regex::find_at(self, input.as_dangerous(), maybe_reject) {
                Some(m) if input.len() == m.end() => return None,
                Some(m) => {
                    maybe_reject = m.end();
                }
                None => return Some(maybe_reject),
            }
        }
    }
}
