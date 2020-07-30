#[test]
fn input_display_valid_utf8() {
    let input = dangerous::input("hello ♥".as_bytes());

    assert_eq!(format!("{}", input), "<68 65 6c 6c 6f 20 e2 99 a5>");
    assert_eq!(format!("{:#}", input), r#""hello ♥""#);
    // max length 2
    assert_eq!(format!("{:.2}", input), "<68 65 ..>");
    assert_eq!(format!("{:#.2}", input), r#""he.." (truncated)"#);
}

#[test]
fn input_display_high_range_utf8() {
    let input = dangerous::input("♥♥".as_bytes());

    assert_eq!(format!("{}", input), "<e2 99 a5 e2 99 a5>");
    assert_eq!(format!("{:#}", input), r#""♥♥""#);
    // max length 1
    assert_eq!(format!("{:.1}", input), "<e2 ..>");
    assert_eq!(format!("{:#.1}", input), r#""♥.." (truncated)"#);
}

#[test]
fn input_display_invalid_utf8() {
    let input = dangerous::input(&[0xFF, b'a', 0xFF]);

    assert_eq!(format!("{}", input), "<ff 61 ff>");
    assert_eq!(format!("{:#}", input), "<ff 'a' ff>");
    // max length 2
    assert_eq!(format!("{:.2}", input), "<ff 61 ..>");
    assert_eq!(format!("{:#.2}", input), "<ff 'a' ..>");
}
