#[macro_use]
mod common;

#[test]
fn valid_utf8() {
    assert_input_display_eq!("hello ♥", "{}", "[68 65 6c 6c 6f 20 e2 99 a5]");
    assert_input_display_eq!("hello ♥", "{:#}", r#""hello ♥""#);
    assert_input_display_eq!("hello ♥", "{:.18}", "[68 65 .. 99 a5]");
    assert_input_display_eq!("hello ♥", "{:#.18}", r#""hello ♥""#);
    assert_input_display_eq!("oh, hello world! ♥", "{:#.16}", r#""oh, " .. "d! ♥""#);
}

#[test]
fn high_range_utf8() {
    assert_input_display_eq!("♥♥♥", "{}", "[e2 99 a5 e2 99 a5 e2 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#}", r#""♥♥♥""#);
    assert_input_display_eq!("♥♥♥", "{:.16}", "[e2 99 .. 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#.16}", r#""♥♥♥""#);
    assert_input_display_eq!("♥♥♥", "{:.19}", "[e2 99 a5 .. 99 a5]");
    assert_input_display_eq!("♥♥♥", "{:#.19}", r#""♥♥♥""#);
}

#[test]
fn invalid_utf8() {
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
