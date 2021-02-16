#[macro_use]
mod common;

use common::*;

#[test]
fn test_of_bytes() {
    let parent = &[1, 2, 3, 4][..];
    let sub = &parent[1..2];
    assert_eq!(Span::from(sub).of(parent).unwrap(), sub);

    let non_span = Span::from(&[1, 2, 2, 4][..]);
    assert_eq!(non_span.of(parent), None);
}

#[test]
fn test_of_str() {
    let parent = "1234";

    let sub = &parent[1..2];
    assert_eq!(Span::from(sub).of(parent).unwrap(), sub);

    let non_span = Span::from("1224");
    assert_eq!(non_span.of(parent), None);
}

#[test]
fn test_of_str_invalid() {
    let parent = "♥♥";

    let sub = &parent[0..3];
    assert_eq!(Span::from(sub).of(parent).unwrap(), sub);

    let non_span = Span::from(&parent.as_bytes()[0..1]);
    assert_eq!(non_span.of(parent), None);
}

#[test]
fn test_of_input_bytes() {
    let parent = dangerous::input(&[1, 2, 3, 4]);
    let sub = &parent.as_dangerous()[1..2];
    assert_eq!(Span::from(sub).of(parent.clone()).unwrap(), sub);

    let non_span = Span::from(&[1, 2, 2, 4][..]);
    assert_eq!(non_span.of(parent), None);
}

#[test]
fn test_of_input_string() {
    let parent = dangerous::input("1234");

    let sub = &parent.as_dangerous()[1..2];
    assert_eq!(Span::from(sub).of(parent.clone()).unwrap(), sub);

    let non_span = Span::from("1224");
    assert_eq!(non_span.of(parent), None);
}

#[test]
fn test_of_input_string_invalid() {
    let parent = dangerous::input("♥♥");

    let sub = &parent.as_dangerous()[0..3];
    assert_eq!(Span::from(sub).of(parent.clone()).unwrap(), sub);

    let non_span = Span::from(&parent.as_dangerous().as_bytes()[0..1]);
    assert_eq!(non_span.of(parent), None);
}
