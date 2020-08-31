use dangerous::{Error, Expected};

#[test]
fn test_utf8_expected_lengths() {
    // Length 1
    dangerous::input(&[0b0111_1111])
        .to_dangerous_str::<Expected>()
        .unwrap();
    // Length 2
    let err = dangerous::input(&[0b1101_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), Some(1));
    // Length 3
    let err = dangerous::input(&[0b1110_1111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), Some(2));
    // Invalid
    let err = dangerous::input(&[0b1111_0111])
        .to_dangerous_str::<Expected>()
        .unwrap_err();
    assert_eq!(err.can_continue_after(), None);
}
