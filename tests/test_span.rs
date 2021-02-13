#[macro_use]
mod common;

use common::*;

#[test]
fn test_is_within() {
    let bytes = [0u8; 64];

    // Within
    let parent = Span::from(&bytes[16..32]);
    let child = Span::from(&bytes[20..24]);
    assert!(child.is_within(parent));
    assert!(parent.is_within(parent));

    // Left out of bound
    let parent = Span::from(&bytes[16..32]);
    let child = Span::from(&bytes[15..24]);
    assert!(!child.is_within(parent));

    // Right out of bound
    let parent = Span::from(&bytes[16..32]);
    let child = Span::from(&bytes[20..33]);
    assert!(!child.is_within(parent));

    // Both out of bound
    let parent = Span::from(&bytes[16..32]);
    let child = Span::from(&bytes[15..33]);
    assert!(!child.is_within(parent));
}
