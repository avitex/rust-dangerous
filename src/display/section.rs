// | format    | str          | bytes        | bytes-ascii    |
// | --------- | ------------ | ------------ | -------------- |
// | head      | `"a" ..`     | `[97 ..]`    | `['a' ..]`     |
// | tail      | `.. "a"`     | `[.. 97]`    | `[.. 'a']`     |
// | head-tail | `"a" .. "a"` | `[97 .. 97]` | `['a' .. 'a']` |
// | span      | `.. "a" ..`  | `[.. 97 ..]` | `[.. 'a' ..]`  |

use core::str;

use crate::fmt::{self, Write};
use crate::util::alt_iter::{Alternate, AlternatingIter};
use crate::util::{slice, unwrap_ok_infallible, utf8};

use super::input::{InputWriter, PreferredFormat};
use super::section_unit::{ByteSectionUnitIter, CharSectionUnitIter, SectionUnit, SectionUnitIter};

const MIN_WIDTH: usize = 16;
const SPACE_COST: usize = 1;
const DELIM_PAIR_COST: usize = 2;
const SIDE_HAS_MORE_COST: usize = ".. ".len();
const HEAD_TAIL_HAS_MORE_COST: usize = SIDE_HAS_MORE_COST;
const STR_HEAD_TAIL_HAS_MORE_COST: usize = SIDE_HAS_MORE_COST + DELIM_PAIR_COST + SPACE_COST;

#[derive(Copy, Clone)]
pub(super) enum SectionOpt<'a> {
    Full,
    Head { width: usize },
    Tail { width: usize },
    HeadTail { width: usize },
    Span { width: usize, span: &'a [u8] },
}

impl<'a> SectionOpt<'a> {
    pub(super) fn compute(self, input: &'a [u8], format: PreferredFormat) -> Section<'a> {
        match self {
            Self::Full => Section::from_full(input, format),
            Self::Head { width } => Section::from_head(input, width, format),
            Self::Tail { width } => Section::from_tail(input, width, format),
            Self::HeadTail { width } => Section::from_head_tail(input, width, format),
            Self::Span { width, span } => Section::from_span(input, span, width, format),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq))]
enum Visible<'a> {
    // head-str, tail-str, span-str
    Str(&'a str),
    // head-str, tail-str, span-str
    StrCjk(&'a str),
    // head-bytes, tail-bytes, span-bytes
    Bytes(&'a [u8]),
    // head-bytes-ascii, tail-bytes-ascii, span-bytes-ascii
    BytesAscii(&'a [u8]),
    // head-tail-str
    StrPair(&'a str, &'a str),
    // head-tail-str
    StrCjkPair(&'a str, &'a str),
    // head-tail-bytes
    BytesPair(&'a [u8], &'a [u8]),
    // head-tail-bytes-ascii
    BytesAsciiPair(&'a [u8], &'a [u8]),
}

pub(super) struct Section<'a> {
    full: &'a [u8],
    visible: Visible<'a>,
    span: Option<&'a [u8]>,
}

impl<'a> Section<'a> {
    pub(super) fn from_full(full: &'a [u8], format: PreferredFormat) -> Self {
        let visible = match format {
            PreferredFormat::Bytes => Visible::Bytes(full),
            PreferredFormat::BytesAscii => Visible::BytesAscii(full),
            PreferredFormat::Str => {
                if let Ok(s) = str::from_utf8(full) {
                    Visible::Str(s)
                } else {
                    Visible::BytesAscii(full)
                }
            }
            PreferredFormat::StrCjk => {
                if let Ok(s) = str::from_utf8(full) {
                    Visible::StrCjk(s)
                } else {
                    Visible::BytesAscii(full)
                }
            }
        };
        Self {
            full,
            visible,
            span: None,
        }
    }

    pub(super) fn from_head(full: &'a [u8], width: usize, format: PreferredFormat) -> Self {
        let width = init_width(width);
        let visible = match format {
            PreferredFormat::Bytes => take_bytes_head(full, width, false),
            PreferredFormat::BytesAscii => take_bytes_head(full, width, true),
            PreferredFormat::Str => take_str_head(full, width, false),
            PreferredFormat::StrCjk => take_str_head(full, width, true),
        };
        Self {
            full,
            visible,
            span: None,
        }
    }

    pub(super) fn from_tail(full: &'a [u8], width: usize, format: PreferredFormat) -> Self {
        let width = init_width(width);
        let visible = match format {
            PreferredFormat::Bytes => take_bytes_tail(full, width, false),
            PreferredFormat::BytesAscii => take_bytes_tail(full, width, true),
            PreferredFormat::Str => take_str_tail(full, width, false),
            PreferredFormat::StrCjk => take_str_tail(full, width, true),
        };
        Self {
            full,
            visible,
            span: None,
        }
    }

    pub(super) fn from_head_tail(full: &'a [u8], width: usize, format: PreferredFormat) -> Self {
        let width = init_width(width);
        let visible = match format {
            PreferredFormat::Bytes => take_bytes_head_tail(full, width, false),
            PreferredFormat::BytesAscii => take_bytes_head_tail(full, width, true),
            PreferredFormat::Str => take_str_head_tail(full, width, false),
            PreferredFormat::StrCjk => take_str_head_tail(full, width, true),
        };
        Self {
            full,
            visible,
            span: None,
        }
    }

    pub(super) fn from_span(
        full: &'a [u8],
        mut span: &'a [u8],
        width: usize,
        format: PreferredFormat,
    ) -> Self {
        if !slice::is_sub_slice(full, span) {
            return Self::from_head_tail(full, width, format);
        }
        let width = init_width(width);
        let full_bounds = full.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        let span_offset = span_bounds.start as usize - full_bounds.start as usize;
        if span.is_empty() {
            if full_bounds.start == span_bounds.start {
                let visible = match format {
                    PreferredFormat::Bytes => take_bytes_head(full, width, false),
                    PreferredFormat::BytesAscii => take_bytes_head(full, width, true),
                    PreferredFormat::Str => take_str_head(full, width, false),
                    PreferredFormat::StrCjk => take_str_head(full, width, true),
                };
                return Self {
                    full,
                    visible,
                    span: Some(span),
                };
            } else if full_bounds.end == span_bounds.end {
                let visible = match format {
                    PreferredFormat::Bytes => take_bytes_tail(full, width, false),
                    PreferredFormat::BytesAscii => take_bytes_tail(full, width, true),
                    PreferredFormat::Str => take_str_tail(full, width, false),
                    PreferredFormat::StrCjk => take_str_tail(full, width, true),
                };
                return Self {
                    full,
                    visible,
                    span: Some(span),
                };
            }
            span = &full[span_offset..=span_offset];
        }
        // If the span starts at an invalid UTF-8 boundary, show the section
        // as bytes-ascii
        let format = match format {
            PreferredFormat::Str | PreferredFormat::StrCjk => {
                if utf8::char_len(full[span_offset]) > 0 {
                    format
                } else {
                    PreferredFormat::BytesAscii
                }
            }
            _ => format,
        };
        let visible = match format {
            PreferredFormat::Bytes => take_bytes_span(full, span_offset, width, false),
            PreferredFormat::BytesAscii => take_bytes_span(full, span_offset, width, true),
            PreferredFormat::Str => take_str_span(full, span_offset, width, false),
            PreferredFormat::StrCjk => take_str_span(full, span_offset, width, true),
        };
        Self {
            full,
            visible,
            span: Some(span),
        }
    }

    pub(super) fn write(&self, w: &mut dyn Write, underline: bool) -> fmt::Result {
        let mut writer = InputWriter::new(w, self.full, self.span, underline);
        match self.visible {
            Visible::Bytes(bytes) => writer.write_bytes_side(bytes, false),
            Visible::BytesAscii(bytes) => writer.write_bytes_side(bytes, true),
            Visible::Str(s) => writer.write_str_side(s, false),
            Visible::StrCjk(s) => writer.write_str_side(s, true),
            Visible::BytesPair(left, right) => writer.write_bytes_sides(left, right, false),
            Visible::BytesAsciiPair(left, right) => writer.write_bytes_sides(left, right, true),
            Visible::StrPair(left, right) => writer.write_str_sides(left, right, false),
            Visible::StrCjkPair(left, right) => writer.write_str_sides(left, right, true),
        }
    }
}

impl<'a> Clone for Section<'a> {
    fn clone(&self) -> Self {
        Self {
            visible: self.visible.clone(),
            ..*self
        }
    }
}

fn init_width(width: usize) -> usize {
    // account for `[]` or `""`
    if width < MIN_WIDTH {
        MIN_WIDTH - DELIM_PAIR_COST
    } else {
        width.saturating_sub(DELIM_PAIR_COST)
    }
}

fn take_str_span(bytes: &[u8], span_offset: usize, width: usize, cjk: bool) -> Visible<'_> {
    let iter = CharSectionUnitIter::new(bytes, cjk);
    if let Ok((start, end)) = take_span(iter, span_offset, width, false) {
        // SAFETY: all chars are checked from the char iterator
        let s = unsafe { utf8::from_unchecked(&bytes[start..end]) };
        if cjk {
            Visible::StrCjk(s)
        } else {
            Visible::Str(s)
        }
    } else {
        take_bytes_span(bytes, span_offset, width, true)
    }
}

fn take_bytes_span(
    bytes: &[u8],
    span_offset: usize,
    width: usize,
    show_ascii: bool,
) -> Visible<'_> {
    let iter = ByteSectionUnitIter::new(bytes, show_ascii);
    let result = take_span(iter, span_offset, width, true);
    let (start, end) = unwrap_ok_infallible(result);
    if show_ascii {
        Visible::BytesAscii(&bytes[start..end])
    } else {
        Visible::Bytes(&bytes[start..end])
    }
}

fn take_str_head(bytes: &[u8], width: usize, cjk: bool) -> Visible<'_> {
    let iter = CharSectionUnitIter::new(bytes, cjk);
    if let Ok((len, _)) = take_head(iter, width, false) {
        // SAFETY: all chars are checked from the char iterator
        let s = unsafe { utf8::from_unchecked(&bytes[..len]) };
        if cjk {
            Visible::StrCjk(s)
        } else {
            Visible::Str(s)
        }
    } else {
        take_bytes_head(bytes, width, true)
    }
}

fn take_bytes_head(bytes: &[u8], width: usize, show_ascii: bool) -> Visible<'_> {
    let iter = ByteSectionUnitIter::new(bytes, show_ascii);
    let result = take_head(iter, width, true);
    let (len, _) = unwrap_ok_infallible(result);
    if show_ascii {
        Visible::BytesAscii(&bytes[..len])
    } else {
        Visible::Bytes(&bytes[..len])
    }
}

fn take_str_tail(bytes: &[u8], width: usize, cjk: bool) -> Visible<'_> {
    let iter = CharSectionUnitIter::new(bytes, cjk);
    if let Ok((len, _)) = take_tail(iter, width, false) {
        let offset = bytes.len() - len;
        // SAFETY: all chars are checked from the char iterator
        let s = unsafe { utf8::from_unchecked(&bytes[offset..]) };
        if cjk {
            Visible::StrCjk(s)
        } else {
            Visible::Str(s)
        }
    } else {
        take_bytes_tail(bytes, width, true)
    }
}

fn take_bytes_tail(bytes: &[u8], width: usize, show_ascii: bool) -> Visible<'_> {
    let iter = ByteSectionUnitIter::new(bytes, show_ascii);
    let result = take_tail(iter, width, true);
    let (len, _) = unwrap_ok_infallible(result);
    let offset = bytes.len() - len;
    if show_ascii {
        Visible::BytesAscii(&bytes[offset..])
    } else {
        Visible::Bytes(&bytes[offset..])
    }
}

fn take_str_head_tail(bytes: &[u8], width: usize, cjk: bool) -> Visible<'_> {
    let iter = CharSectionUnitIter::new(bytes, cjk);
    if let Ok((start, end)) = take_head_tail(iter, width, false, STR_HEAD_TAIL_HAS_MORE_COST) {
        // SAFETY: all chars are checked from the char iterator
        unsafe {
            if start == end {
                let s = utf8::from_unchecked(&bytes[..]);
                if cjk {
                    return Visible::StrCjk(s);
                }
                return Visible::Str(s);
            }
            let left = utf8::from_unchecked(&bytes[..start]);
            let right = utf8::from_unchecked(&bytes[end..]);
            if cjk {
                return Visible::StrCjkPair(left, right);
            }
            return Visible::StrPair(left, right);
        }
    }
    take_bytes_head_tail(bytes, width, true)
}

fn take_bytes_head_tail(bytes: &[u8], width: usize, show_ascii: bool) -> Visible<'_> {
    let iter = ByteSectionUnitIter::new(bytes, show_ascii);
    let result = take_head_tail(iter, width, true, HEAD_TAIL_HAS_MORE_COST);
    let (start, end) = unwrap_ok_infallible(result);
    if start == end {
        if show_ascii {
            Visible::BytesAscii(&bytes[..])
        } else {
            Visible::Bytes(&bytes[..])
        }
    } else {
        let left = &bytes[..start];
        let right = &bytes[end..];
        if show_ascii {
            Visible::BytesAsciiPair(left, right)
        } else {
            Visible::BytesPair(left, right)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// Returns `Result<(length, remaining), ()>`
fn take_head<I, E>(iter: I, width: usize, space_separated: bool) -> Result<(usize, usize), E>
where
    I: SectionUnitIter<E>,
{
    take_side(iter, width, space_separated)
}

/// Returns `Result<(length, remaining), ()>`
fn take_tail<I, E>(iter: I, width: usize, space_separated: bool) -> Result<(usize, usize), E>
where
    I: SectionUnitIter<E>,
{
    take_side(iter.rev(), width, space_separated)
}

/// Returns `Result<(length, remaining), ()>`
fn take_side<I, E>(iter: I, width: usize, space_separated: bool) -> Result<(usize, usize), E>
where
    I: Iterator<Item = Result<SectionUnit, E>>,
{
    elements_to_fit_width(
        iter,
        0,
        width,
        space_separated,
        SIDE_HAS_MORE_COST,
        |len, element_result| match element_result {
            Ok(element) => Ok((len + element.len_utf8, element.display_cost)),
            Err(e) => Err(e),
        },
    )
}

/// Returns `Result<(start, end), ()>`
fn take_head_tail<I, E>(
    iter: I,
    width: usize,
    space_separated: bool,
    has_more_cost: usize,
) -> Result<(usize, usize), E>
where
    I: SectionUnitIter<E>,
{
    let total_len = iter.as_slice().len();
    let ((front_len, back_offset), _) = elements_to_fit_width(
        AlternatingIter::front(iter),
        (0, total_len),
        width,
        space_separated,
        has_more_cost,
        |(front_len, back_offset), alt_element| match alt_element {
            Alternate::Front(Ok(element)) => Ok((
                (front_len + element.len_utf8, back_offset),
                element.display_cost,
            )),
            Alternate::Back(Ok(element)) => Ok((
                (front_len, back_offset - element.len_utf8),
                element.display_cost,
            )),
            Alternate::Front(Err(err)) | Alternate::Back(Err(err)) => Err(err),
        },
    )?;
    Ok((front_len, back_offset))
}

/// Returns `Result<(start, end), ()>`
fn take_span<I, E>(
    iter: I,
    span_offset: usize,
    width: usize,
    space_separated: bool,
) -> Result<(usize, usize), E>
where
    I: SectionUnitIter<E>,
{
    // Attempt to get 1/3 of the total width before the span.
    let init_backward_width = width / 3 + SIDE_HAS_MORE_COST;
    let backward_offset = iter.as_slice().len() - span_offset;
    let init_backward_iter = iter.clone().skip_tail_bytes(backward_offset);
    let (init_head_len, head_remaining_width) =
        take_tail(init_backward_iter, init_backward_width, space_separated)?;
    // Attempt to get 2/3 plus what couldn't be taken from before.
    let forward_width = width
        .saturating_sub(init_backward_width)
        .saturating_add(head_remaining_width);
    let forward_iter = iter.clone().skip_head_bytes(span_offset);
    let (tail_len, tail_remaining_width) = take_head(forward_iter, forward_width, space_separated)?;
    // If we had some remaining width from the span onwards, see if we can use it before.
    let head_len = if tail_remaining_width > 0 {
        let backward_iter = iter.skip_tail_bytes(backward_offset);
        let backward_width = width
            .saturating_sub(forward_width)
            .saturating_add(tail_remaining_width);
        let (head_len, _) = take_tail(backward_iter, backward_width, space_separated)?;
        head_len
    } else {
        init_head_len
    };
    Ok((span_offset - head_len, span_offset + tail_len))
}

///////////////////////////////////////////////////////////////////////////////

fn elements_to_fit_width<A, I, F, E>(
    mut iter: I,
    mut acc: A,
    width: usize,
    space_separated: bool,
    has_more_cost: usize,
    mut element_fn: F,
) -> Result<(A, usize), E>
where
    A: Copy,
    I: Iterator,
    F: FnMut(A, I::Item) -> Result<(A, usize), E>,
{
    let mut is_first = true;
    let mut budget = width;
    while let Some(item) = iter.next() {
        // Try to get the next element
        let (next_acc, display_cost) = match element_fn(acc, item) {
            Ok((next_acc, display_cost)) => (next_acc, display_cost),
            Err(err) => return Err(err),
        };
        // Handle the next element
        let has_next = iter.size_hint().0 > 0;
        // Make sure we have room for the separator if any
        let element_cost = if space_separated {
            if is_first {
                is_first = false;
                display_cost
            } else {
                SPACE_COST + display_cost
            }
        } else {
            display_cost
        };
        // Make sure we have room for the has more
        let required = if has_next {
            element_cost + has_more_cost
        } else {
            element_cost
        };
        // Check that we have enough room for the this element, and if so
        // subtract it.
        if budget >= required {
            // We did, update the budget and commit the accumulator.
            budget -= element_cost;
            acc = next_acc;
        } else {
            if has_next {
                budget = budget.saturating_sub(has_more_cost)
            }
            break;
        }
    }
    Ok((acc, budget))
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    use crate::display::InputDisplay;
    use crate::input;

    const BAD_UTF8: u8 = 0b1101_1111; // 223, 0xdf

    macro_rules! assert_computed_visible_eq {
        (from_head, {
            input: $input:expr,
            format: $format:expr,
            visible: $visible:expr,
            display: $display:expr,
        }) => {
            assert_computed_visible_eq!(from_head, head, {
                input: $input,
                format: $format,
                visible: $visible,
                display: $display,
            });
        };
        (from_tail, {
            input: $input:expr,
            format: $format:expr,
            visible: $visible:expr,
            display: $display:expr,
        }) => {
            assert_computed_visible_eq!(from_tail, tail, {
                input: $input,
                format: $format,
                visible: $visible,
                display: $display,
            });
        };
        (from_head_tail, {
            input: $input:expr,
            format: $format:expr,
            visible: $visible:expr,
            display: $display:expr,
        }) => {
            assert_computed_visible_eq!(from_head_tail, head_tail, {
                input: $input,
                format: $format,
                visible: $visible,
                display: $display,
            });
        };
        ($from:ident, $input_section:ident, {
            input: $input:expr,
            format: $format:expr,
            visible: $visible:expr,
            display: $display:expr,
        }) => {{
            let full = $input;
            let section = Section::$from($input, $display.len(), $format);
            let input = InputDisplay::new(&input(&full[..]))
                .format($format)
                .$input_section($display.len());
            assert_eq!(section.visible, $visible);
            assert_eq!($display, input.to_string());
        }};
    }

    macro_rules! assert_computed_span_visible_eq {
        ({
            input: $input:expr,
            span: $range:expr,
            format: $format:expr,
            visible: $visible:expr,
            display: $display:expr,
        }) => {{
            let full = $input;
            let span = &full[$range];
            let section = Section::from_span(full, span, $display.len(), $format);
            let input = InputDisplay::new(&input(&full[..]))
                .format($format)
                .span(&input(span), $display.len());
            assert_eq!(section.visible, $visible);
            assert_eq!($display, input.to_string());
        }};
    }

    ///////////////////////////////////////////////////////////////////////////
    // Section head tests

    #[test]
    fn test_computed_head_str_single() {
        assert_computed_visible_eq!(from_head, {
            input: b"a",
            format: PreferredFormat::Str,
            visible: Visible::Str("a"),
            display: r#""a""#,
        });
    }

    #[test]
    fn test_computed_head_str_short_non_ascii() {
        assert_computed_visible_eq!(from_head, {
            input: &[b'a', b'b', 3, 3],
            format: PreferredFormat::Str,
            visible: Visible::Str("ab\u{3}\u{3}"),
            display: "\"ab\\u{3}\\u{3}\"",
        });
    }

    #[test]
    fn test_computed_head_str_short_non_ascii_bad_utf8() {
        assert_computed_visible_eq!(from_head, {
            input: &[b'a', b'b', 3, BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(&[b'a', b'b', 3, BAD_UTF8]),
            display: r#"['a' 'b' 03 df]"#,
        });
    }

    #[test]
    fn test_computed_head_str_min_width() {
        assert_computed_visible_eq!(from_head, {
            input: b"abcdefghijklmnopqrstuvwxyz",
            format: PreferredFormat::Str,
            visible: Visible::Str("abcdefghijk"),
            display: r#""abcdefghijk" .."#,
        });
    }

    #[test]
    fn test_computed_head_str_short_bad_utf8() {
        assert_computed_visible_eq!(from_head, {
            input: &[b'a', b'b', b'c', b'd', b'e', BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(b"abc"),
            display: r#"['a' 'b' 'c' ..]"#,
        });
    }

    #[test]
    fn test_computed_head_str_min_width_bad_utf8() {
        // FIXME: make this better (aka output str instead)?
        assert_computed_visible_eq!(from_head, {
            input: &[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(b"abc"),
            display: r#"['a' 'b' 'c' ..]"#,
        });
    }

    #[test]
    fn test_computed_head_str_over_min_width_bad_utf8() {
        assert_computed_visible_eq!(from_head, {
            input: &[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::Str("abcdefghijk"),
            display: r#""abcdefghijk" .."#,
        });
    }

    ///////////////////////////////////////////////////////////////////////////
    // Section tail tests

    #[test]
    fn test_computed_tail_str_single() {
        assert_computed_visible_eq!(from_tail, {
            input: b"a",
            format: PreferredFormat::Str,
            visible: Visible::Str("a"),
            display: r#""a""#,
        });
    }

    #[test]
    fn test_computed_tail_str_short_non_ascii() {
        assert_computed_visible_eq!(from_tail, {
            input: &[b'a', 3, 3, b'b'],
            format: PreferredFormat::Str,
            visible: Visible::Str("a\u{3}\u{3}b"),
            display: "\"a\\u{3}\\u{3}b\"",
        });
    }

    #[test]
    fn test_computed_tail_str_short_non_ascii_bad_utf8() {
        assert_computed_visible_eq!(from_tail, {
            input: &[b'a', b'b', 3, BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(&[b'a', b'b', 3, BAD_UTF8]),
            display: r#"['a' 'b' 03 df]"#,
        });
    }

    #[test]
    fn test_computed_tail_str_min_width() {
        assert_computed_visible_eq!(from_tail, {
            input: b"abcdefghijklmnopqrstuvwxyz",
            format: PreferredFormat::Str,
            visible: Visible::Str("pqrstuvwxyz"),
            display: r#".. "pqrstuvwxyz""#,
        });
    }

    #[test]
    fn test_computed_tail_str_min_width_bad_utf8() {
        // FIXME: make this better (aka output str instead)?
        assert_computed_visible_eq!(from_tail, {
            input: &[BAD_UTF8, b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z'],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(b"xyz"),
            display: r#"[.. 'x' 'y' 'z']"#,
        });
    }

    #[test]
    fn test_computed_tail_str_over_min_width_bad_utf8() {
        assert_computed_visible_eq!(from_tail, {
            input: &[BAD_UTF8, b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z'],
            format: PreferredFormat::Str,
            visible: Visible::Str("pqrstuvwxyz"),
            display: r#".. "pqrstuvwxyz""#,
        });
    }

    ///////////////////////////////////////////////////////////////////////////
    // Section head-tail tests

    #[test]
    fn test_computed_head_tail_str_single() {
        assert_computed_visible_eq!(from_head_tail, {
            input: b"a",
            format: PreferredFormat::Str,
            visible: Visible::Str("a"),
            display: r#""a""#,
        });
    }

    #[test]
    fn test_computed_head_tail_str_short_non_ascii() {
        assert_computed_visible_eq!(from_head_tail, {
            input: &[b'a', 3, 3, b'b'],
            format: PreferredFormat::Str,
            visible: Visible::Str("a\u{3}\u{3}b"),
            display: "\"a\\u{3}\\u{3}b\"",
        });
    }

    #[test]
    fn test_computed_head_tail_str_non_ascii_bad_utf8() {
        assert_computed_visible_eq!(from_head_tail, {
            input: &[b'a', b'b', 3, BAD_UTF8],
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(&[b'a', b'b', 3, BAD_UTF8]),
            display: r#"['a' 'b' 03 df]"#,
        });
    }

    #[test]
    fn test_computed_head_tail_str_min_width() {
        assert_computed_visible_eq!(from_head_tail, {
            input: b"abcdefghijklmnopqrstuvwxyz",
            format: PreferredFormat::Str,
            visible: Visible::StrPair("abcd", "wxyz"),
            display: r#""abcd" .. "wxyz""#,
        });
    }

    #[test]
    fn test_computed_head_tail_str_min_width_bad_utf8() {
        assert_computed_visible_eq!(from_head_tail, {
            input: &[b'a', b'b', BAD_UTF8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, BAD_UTF8, b'c'],
            format: PreferredFormat::Str,
            visible: Visible::BytesAsciiPair(b"ab", b"c"),
            display: r#"['a' 'b' .. 'c']"#,
        });
    }

    #[test]
    fn test_computed_head_tail_str_over_min_width_bad_utf8() {
        assert_computed_visible_eq!(from_head_tail, {
            input: &[b'a', b'b', b'c', BAD_UTF8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, BAD_UTF8, b'y', b'z'],
            format: PreferredFormat::Str,
            visible: Visible::BytesAsciiPair(b"ab", b"z"),
            display: r#"['a' 'b' .. 'z']"#,
        });
    }

    ///////////////////////////////////////////////////////////////////////////
    // Section tail tests

    #[test]
    fn test_computed_span_str_long_head() {
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //       ^^^
            span: 0..3,
            format: PreferredFormat::Str,
            visible: Visible::Str("abcdefghijkl"),
            //                     ^^^
            display: r#""abcdefghijkl" .."#,
        });
    }

    #[test]
    fn test_computed_span_str_long_tail() {
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                              ^^^
            span: 23..26,
            format: PreferredFormat::Str,
            visible: Visible::Str("pqrstuvwxyz"),
            //                         ^^^
            display: r#".. "pqrstuvwxyz""#,
        });
    }

    #[test]
    fn test_computed_span_str_over_min_tail() {
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                        ^^^^^^^^^
            span: 18..26,
            format: PreferredFormat::Str,
            visible: Visible::Str("opqrstuv"),
            //                        ^^^^^
            display: r#".. "opqrstuv" .."#,
        });
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                        ^^^^^^^^^
            span: 18..26,
            format: PreferredFormat::Str,
            visible: Visible::Str("mnopqrstuvwx"),
            display: r#".. "mnopqrstuvwx" .."#,
        });
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                        ^^^^^^^^^
            span: 18..26,
            format: PreferredFormat::Str,
            visible: Visible::Str("klmnopqrstuvwxyz"),
            //                            ^^^^^^^^^
            display: r#".. "klmnopqrstuvwxyz""#,
        });
    }

    #[test]
    fn test_computed_span_str_long_middle() {
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                   ^^
            span: 12..14,
            format: PreferredFormat::Str,
            visible: Visible::Str("fghijklmnopqrstuv"),
            //                            ^^
            display: r#".. "fghijklmnopqrstuv" .."#,
        });
    }

    #[test]
    fn test_computed_span_str_last() {
        assert_computed_span_visible_eq!({
            input: b"abcdefghijklmnopqrstuvwxyz",
            //                                ^
            span: 25..26,
            format: PreferredFormat::Str,
            visible: Visible::Str("jklmnopqrstuvwxyz"),
            //                                     ^
            display: r#".. "jklmnopqrstuvwxyz""#,
        });
    }

    #[test]
    fn test_computed_span_str_last_bad_utf8() {
        assert_computed_span_visible_eq!({
            input: &[b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', BAD_UTF8],
            //                                                                         ^
            span: 11..12,
            format: PreferredFormat::Str,
            visible: Visible::BytesAscii(&[b'x', b'y', b'z', BAD_UTF8]),
            //                                               ^
            display: r#"[.. 'x' 'y' 'z' df]"#,
        });
    }

    #[test]
    fn test_computed_span_bytes_last() {
        assert_computed_span_visible_eq!({
            input: &[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
            //                                     ^
            span: 5..,
            format: PreferredFormat::BytesAscii,
            visible: Visible::BytesAscii(&[0xcc, 0xdd, 0xee, 0xff]),
            //                                               ^
            display: r#"[.. cc dd ee ff]"#,
        });
    }
}
