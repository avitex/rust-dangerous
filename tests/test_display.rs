#[macro_use]
mod common;

use common::*;
use std::fmt;

#[test]
fn test_valid_utf8() {
    assert_input_display_eq!("hello ♥", "{}", "[68 65 6c 6c 6f 20 e2 99 a5]");
    assert_input_display_eq!("hello ♥", "{:#}", r#""hello ♥""#);
    assert_input_display_eq!("hello ♥", "{:.18}", "[68 65 .. 99 a5]");
    assert_input_display_eq!("hello ♥", "{:#.18}", r#""hello ♥""#);
    assert_input_display_eq!("oh, hello world! ♥", "{:#.16}", r#""oh, " .. "d! ♥""#);
}

#[test]
fn test_high_range_utf8() {
    assert_input_display_eq!("♥♥♥", "{}", "[e2 99 a5 e2 99 a5 e2 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#}", r#""♥♥♥""#);
    assert_input_display_eq!("♥♥♥", "{:.16}", "[e2 99 .. 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#.16}", r#""♥♥♥""#);
    assert_input_display_eq!("♥♥♥", "{:.19}", "[e2 99 a5 .. 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#.19}", r#""♥♥♥""#);
}

#[test]
fn test_invalid_utf8() {
    assert_input_display_eq!(&[0xFF, 0xFF, b'a'], "{}", "[ff ff 61]");
    assert_input_display_eq!(&[0xFF, 0xFF, b'a'], "{:#}", "[ff ff 'a']");
    assert_input_display_eq!(
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, b'a'],
        "{:.18}",
        "[ff ff .. ff 61]"
    );
    assert_input_display_eq!(
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, b'a'],
        "{:#.18}",
        "[ff ff .. ff 'a']"
    );
}

#[test]
fn test_invalid_span_does_nothing() {
    let display = input!(b"hello").display().span(&input!("world"), 16);
    assert_eq!(display.to_string(), "[68 65 6c 6c 6f]");
    assert_eq!(display.underline(true).to_string(), "                ");
    let display = input!(b"hello").display().span(&input!(""), 16);
    assert_eq!(display.to_string(), "[68 65 6c 6c 6f]");
    assert_eq!(display.underline(true).to_string(), "                ");
}

#[test]
fn test_format_with_mut_ref_write() {
    use dangerous::display::{DisplayBase, Write};

    struct Helper;

    impl DisplayBase for Helper {
        fn fmt(&self, w: &mut dyn Write) -> fmt::Result {
            Write::write_str(w, "a")?;
            Write::write_char(w, ',')?;
            Write::write_usize(w, 1)?;
            Write::write_char(w, ',')?;
            Write::write_hex(w, 1)?;
            Write::write_char(w, ',')?;
            Write::write_hex(w, 128)?;
            Write::write_char(w, ',')?;
            Write::write_hex(w, 129)
        }
    }

    impl fmt::Display for Helper {
        fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
            DisplayBase::fmt(&&self, &mut f)
        }
    }

    assert_eq!(Helper.to_string(), "a,1,01,80,81");
}
