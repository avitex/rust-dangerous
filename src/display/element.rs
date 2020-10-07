#[cfg(feature = "unicode")]
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Copy)]
pub(super) struct Element {
    pub(super) len_utf8: usize,
    pub(super) display_cost: usize,
}

impl Element {
    pub(super) fn byte(byte: u8, show_ascii: bool) -> Self {
        let display_cost = if show_ascii && byte.is_ascii_graphic() {
            match byte {
                b'\'' | b'\n' | b'\r' => r#"'\x'"#.len(),
                _ => r#"'x'"#.len(),
            }
        } else {
            b"xx".len()
        };
        Self {
            display_cost,
            len_utf8: 1,
        }
    }

    pub(super) fn unicode(c: char, cjk: bool) -> Self {
        match c {
            '"' => Self {
                display_cost: r#"\""#.len(),
                len_utf8: 1,
            },
            '\n' | '\r' => Self {
                display_cost: r#"\x"#.len(),
                len_utf8: 1,
            },
            '\0' => Self {
                display_cost: unicode_escape_len(c),
                len_utf8: 1,
            },
            c => {
                let display_cost = match unicode_width(c, cjk) {
                    Some(width) => width,
                    None => unicode_escape_len(c),
                };
                Self {
                    display_cost,
                    len_utf8: c.len_utf8(),
                }
            }
        }
    }
}

#[cfg(feature = "unicode")]
#[inline]
fn unicode_width(c: char, cjk: bool) -> Option<usize> {
    if cjk {
        c.width_cjk()
    } else {
        c.width()
    }
}

#[cfg(not(feature = "unicode"))]
#[inline]
fn unicode_width(_c: char, _cjk: bool) -> Option<usize> {
    Some(1)
}

#[inline]
fn unicode_escape_len(c: char) -> usize {
    "\\u{}".len() + count_digits(c as u32)
}

fn count_digits(mut num: u32) -> usize {
    let mut count = 1;
    while num > 9 {
        count += 1;
        num /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_escape_len() {
        assert_eq!("\\u{1}".len(), unicode_escape_len('\u{1}'));
        assert_eq!("\\u{11}".len(), unicode_escape_len('\u{11}'));
    }
}
