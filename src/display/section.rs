use crate::input::{input, Input};
use crate::string::{utf8_char_display_width, CharIter};

#[derive(Copy, Clone)]
pub enum SectionOption<'i> {
    Tail { width: usize },
    Head { width: usize },
    HeadTail { width: usize },
    Span { span: &'i Input, width: usize },
}

pub(crate) enum SectionPart<'i> {
    Input(&'i [u8]),
    Span(&'i [u8]),
}

#[derive(Clone)]
pub(crate) struct Section<'i> {
    input: &'i Input,
    section: &'i Input,
    section_is_str: bool,
    span: Option<&'i Input>,
}

impl<'i> Section<'i> {
    pub(crate) fn compute(opt: SectionOption<'i>, input: &'i Input, str_hint: bool) -> Self {
        match opt {
            SectionOption::Head { width } => Self::from_head(input, width, str_hint),
            SectionOption::Tail { width } => Self::from_tail(input, width, str_hint),
            SectionOption::HeadTail { width } => Self::from_head_tail(input, width, str_hint),
            SectionOption::Span { span, width } => Self::from_span(input, span, width, str_hint),
        }
    }

    pub(crate) fn from_head(input: &'i Input, width: usize, str_hint: bool) -> Self {
        if str_hint {
            let (section, section_is_str) = take_str_head_width(input, width, false);
            Self {
                input,
                section,
                section_is_str,
                span: None,
            }
        } else {
            let section = take_bytes_head_width(input, width);
            Self {
                input,
                section,
                section_is_str: false,
                span: None,
            }
        }
    }

    pub(crate) fn from_tail(input: &'i Input, width: usize, str_hint: bool) -> Self {
        unimplemented!()
    }

    pub(crate) fn from_head_tail(input: &'i Input, width: usize, str_hint: bool) -> Self {
        unimplemented!()
    }

    pub(crate) fn from_span(
        input: &'i Input,
        span: &'i Input,
        width: usize,
        str_hint: bool,
    ) -> Self {
        unimplemented!()
    }

    pub(crate) fn is_str(&self) -> bool {
        self.section_is_str
    }

    pub(crate) fn has_more_before(&self) -> bool {
        let input_bounds = self.section.as_dangerous_ptr_range();
        let section_bounds = self.section.as_dangerous_ptr_range();
        input_bounds.start < section_bounds.start
    }

    pub(crate) fn has_more_after(&self) -> bool {
        let input_bounds = self.section.as_dangerous_ptr_range();
        let section_bounds = self.section.as_dangerous_ptr_range();
        input_bounds.end > section_bounds.end
    }

    pub(crate) fn highlight_before(&self) -> bool {
        self.span.map_or(false, |span| {
            let section_bounds = self.section.as_dangerous_ptr_range();
            let span_bounds = span.as_dangerous_ptr_range();
            section_bounds.start > span_bounds.start
        })
    }

    pub(crate) fn highlight_after(&self) -> bool {
        self.span.map_or(false, |span| {
            let section_bounds = self.section.as_dangerous_ptr_range();
            let span_bounds = span.as_dangerous_ptr_range();
            section_bounds.end < span_bounds.end
        })
    }

    pub(crate) fn highlight_open(&self) -> bool {
        self.span
            .map_or(false, |span| self.section.start().ptr_eq(span))
    }

    pub(crate) fn highlight_close(&self) -> bool {
        self.span
            .map_or(false, |span| self.section.end().ptr_eq(span))
    }

    pub(crate) fn parts(&self) -> impl Iterator<Item = SectionPart<'i>> {
        core::iter::from_fn(|| None)
    }
}

fn elements_to_fit_width<T, F>(
    mut acc: T,
    width: usize,
    has_more_len: usize,
    space_separated: bool,
    mut next_element: F,
) -> T
where
    F: FnMut(&T) -> Option<(T, usize, bool)>,
{
    debug_assert!(
        width >= has_more_len,
        "should have enough space for at least one has more"
    );
    let mut is_first = true;
    let mut budget = width;
    while let Some((next_acc, display_width, has_next)) = next_element(&acc) {
        // Make sure we have room for the seperator if any
        let element_cost = if space_separated {
            if is_first {
                is_first = false;
                display_width
            } else {
                display_width + 1
            }
        } else {
            display_width
        };
        // Make sure we have room for the has more
        let required = if has_next {
            element_cost + has_more_len
        } else {
            element_cost
        };
        // Check that we have enough room for the this element,
        // and if so subtract it.
        if budget >= required {
            // We did, update the budget and commit the accumulator.
            budget -= element_cost;
            acc = next_acc;
        } else {
            break;
        }
    }
    acc
}

fn take_bytes_head_width(complete: &Input, width: usize) -> &Input {
    // let bytes = complete.as_dangerous();
    // let max_bytes = width.saturating_sub(1) / 2;
    // input(&bytes[offset..])
    unimplemented!()
}

fn take_byte_str_head_width(complete: &Input, width: usize) -> &Input {
    let bytes = complete.as_dangerous();
    let mut byte_iter = bytes.iter();
    let offset = elements_to_fit_width(0, width, " ..".len(), true, |offset| {
        byte_iter.next().map(|byte| {
            let cost = if byte.is_ascii_graphic() {
                b"'x'".len()
            } else {
                b"ff".len()
            };
            let offset = offset + 1;
            let has_more = !byte_iter.as_slice().is_empty();
            (offset, cost, has_more)
        })
    });
    input(&bytes[offset..])
}

fn take_str_head_width(complete: &Input, width: usize, cjk: bool) -> (&Input, bool) {
    let bytes = complete.as_dangerous();
    let mut char_iter = CharIter::new(complete);
    let mut is_str = true;
    let offset = elements_to_fit_width(0, width, " ..".len(), false, |offset| {
        char_iter.next().and_then(|result| {
            if let Ok(c) = result {
                let cost = utf8_char_display_width(c, cjk);
                let offset = offset + c.len_utf8();
                let has_more = !char_iter.tail().is_empty();
                Some((offset, cost, has_more))
            } else {
                is_str = false;
                None
            }
        })
    });
    if is_str {
        (input(&bytes[offset..]), false)
    } else {
        (take_byte_str_head_width(complete, width), false)
    }
}

fn head_tail_max(max: usize) -> (usize, usize) {
    let half = max / 2;
    (half + max % 2, half)
}
