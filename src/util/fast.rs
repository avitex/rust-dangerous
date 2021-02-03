use crate::util::utf8::CharBytes;

///////////////////////////////////////////////////////////////////////////////
// u8

#[cfg(feature = "bytecount")]
pub(crate) fn count_u8(needle: u8, haystack: &[u8]) -> usize {
    bytecount::count(haystack, needle)
}

#[cfg(not(feature = "bytecount"))]
pub(crate) fn count_u8(needle: u8, haystack: &[u8]) -> usize {
    haystack.iter().copied().filter(|b| *b == needle).count()
}

#[cfg(feature = "memchr")]
pub(crate) fn find_u8_match(needle: u8, haystack: &[u8]) -> Option<usize> {
    memchr::memchr(needle, haystack)
}

#[cfg(not(feature = "memchr"))]
pub(crate) fn find_u8_match(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().copied().position(|b| b == needle)
}

// FIXME: impl SIMD variant
pub(crate) fn find_u8_reject(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().copied().position(|b| b != needle)
}

///////////////////////////////////////////////////////////////////////////////
// char

#[cfg(feature = "bytecount")]
pub(crate) fn num_chars(s: &str) -> usize {
    bytecount::num_chars(s.as_bytes())
}

#[cfg(not(feature = "bytecount"))]
pub(crate) fn num_chars(s: &str) -> usize {
    s.chars().count()
}

pub(crate) fn find_char_match(needle: char, haystack: &[u8]) -> Option<usize> {
    let needle = CharBytes::from(needle);
    find_slice_match(needle.as_bytes(), haystack)
}

pub(crate) fn find_char_reject(needle: char, haystack: &[u8]) -> Option<usize> {
    let needle = CharBytes::from(needle);
    find_slice_reject(needle.as_bytes(), haystack)
}

///////////////////////////////////////////////////////////////////////////////
// slice

#[cfg(feature = "memchr")]
pub(crate) fn find_slice_match(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() || needle.is_empty() {
        return None;
    }
    let needle_first = needle[0];
    if needle.len() == 1 {
        return find_u8_match(needle_first, haystack);
    }
    let mut last_fail = 0;
    loop {
        match find_u8_match(needle_first, &haystack[last_fail..]) {
            None => return None,
            Some(index) if haystack[last_fail + index..needle.len()] == *needle => {
                return Some(index);
            }
            Some(next_fail) => last_fail = next_fail,
        }
    }
}

#[cfg(not(feature = "memchr"))]
pub(crate) fn find_slice_match(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() || needle.is_empty() {
        return None;
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .find_map(|(i, w)| if w == needle { Some(i) } else { None })
}

// FIXME: impl SIMD variant
pub(crate) fn find_slice_reject(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    if haystack.is_empty() || needle.is_empty() {
        return None;
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .find_map(|(i, w)| if w == needle { None } else { Some(i) })
}
