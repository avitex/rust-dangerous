// #[test]
// fn test_span_of() {
//     let parent = dangerous::input(&[1, 2, 3, 4]);
//     let sub_range = 1..2;
//     let sub = dangerous::input(&parent.as_dangerous()[sub_range.clone()]);
//     assert_eq!(sub.span_of(&parent), Some(sub_range));

//     let non_span = dangerous::input(&[1, 2, 2, 4]);
//     assert_eq!(non_span.span_of(&parent), None);
// }

// #[test]
// fn test_span_of_non_empty() {
//     let parent = dangerous::input(&[1, 2, 3, 4]);
//     let sub_range = 1..2;
//     let sub = dangerous::input(&parent.as_dangerous()[sub_range.clone()]);
//     assert_eq!(sub.span_of_non_empty(&parent), Some(sub_range));

//     let non_span = dangerous::input(&[]);
//     assert_eq!(non_span.span_of_non_empty(&parent), None);
// }

// #[test]
// fn test_is_within() {
//     let bytes = [0u8; 64];

//     // Within
//     let parent = input!(&bytes[16..32]);
//     let child = input!(&bytes[20..24]);
//     assert!(child.is_within(&parent));
//     assert!(parent.is_within(&parent));

//     // Left out of bound
//     let parent = input!(&bytes[16..32]);
//     let child = input!(&bytes[15..24]);
//     assert!(!child.is_within(&parent));

//     // Right out of bound
//     let parent = input!(&bytes[16..32]);
//     let child = input!(&bytes[20..33]);
//     assert!(!child.is_within(&parent));

//     // Both out of bound
//     let parent = input!(&bytes[16..32]);
//     let child = input!(&bytes[15..33]);
//     assert!(!child.is_within(&parent));
// }
