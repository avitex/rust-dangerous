macro_rules! display_eq {
    ($input:expr, $format:expr, $expected:expr) => {
        assert_eq!(
            format!($format, dangerous::input($input.as_ref())),
            $expected
        );
    };
}

#[test]
fn valid_utf8() {
    display_eq!("hello ♥", "{}", "[68 65 6c 6c 6f 20 e2 99 a5]");
    display_eq!("hello ♥", "{:#}", r#""hello ♥""#);
    // max length 2
    display_eq!("hello ♥", "{:.2}", "[68 .. a5]");
    display_eq!("hello ♥", "{:#.2}", r#""h".."♥""#);
}

#[test]
fn high_range_utf8() {
    display_eq!("♥♥", "{}", "[e2 99 a5 e2 99 a5]");
    display_eq!("♥♥", "{:#}", r#""♥♥""#);
    // max length 1
    display_eq!("♥♥", "{:.1}", "[e2 ..]");
    display_eq!("♥♥", "{:#.1}", r#""♥".."#);
    // max length 2
    display_eq!("♥♥", "{:.2}", "[e2 .. a5]");
    display_eq!("♥♥", "{:#.2}", r#""♥♥""#);
}

#[test]
fn invalid_utf8() {
    display_eq!(&[0xFF, 0xFF, b'a'], "{}", "[ff ff 61]");
    display_eq!(&[0xFF, 0xFF, b'a'], "{:#}", "[ff ff 'a']");
    // max length 2
    display_eq!(&[0xFF, 0xFF, b'a'], "{:.2}", "[ff .. 61]");
    display_eq!(&[0xFF, 0xFF, b'a'], "{:#.2}", "[ff .. 'a']");
}
