use core::fmt;

use super::unit::{byte_display_width, byte_display_write, char_display_width, char_display_write};

pub(super) struct InputWriter<'a, W>
where
    W: fmt::Write,
{
    w: W,
    underline: bool,
    full: &'a [u8],
    span: Option<&'a [u8]>,
}

impl<'a, W> InputWriter<'a, W>
where
    W: fmt::Write,
{
    pub(super) fn new(w: W, full: &'a [u8], span: Option<&'a [u8]>, underline: bool) -> Self {
        Self {
            w,
            full,
            span,
            underline,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Bytes

    pub(super) fn write_bytes_side(&mut self, side: &[u8], show_ascii: bool) -> fmt::Result {
        self.write_bytes_open(side)?;
        self.write_bytes(side, show_ascii)?;
        self.write_bytes_close(side)
    }

    pub(super) fn write_bytes_sides(
        &mut self,
        left: &[u8],
        right: &[u8],
        show_ascii: bool,
    ) -> fmt::Result {
        self.write_bytes_open(left)?;
        self.write_bytes(left, show_ascii)?;
        self.write_space(1)?;
        self.write_more(is_span_overlapping_end(left, self.span))?;
        self.write_space(1)?;
        self.write_bytes(right, show_ascii)?;
        self.write_bytes_close(right)
    }

    fn write_bytes_open(&mut self, bytes: &[u8]) -> fmt::Result {
        if has_more_before(bytes, self.full) {
            self.write_delim('[', false)?;
            self.write_more(is_span_overlapping_start(bytes, self.span))?;
            self.write_space(1)
        } else {
            self.write_delim('[', is_span_pointing_to_start(bytes, self.span))
        }
    }

    fn write_bytes_close(&mut self, bytes: &[u8]) -> fmt::Result {
        if has_more_after(bytes, self.full) {
            self.write_space(1)?;
            self.write_more(is_span_overlapping_end(bytes, self.span))?;
            self.write_delim(']', false)
        } else {
            self.write_delim(']', is_span_pointing_to_end(bytes, self.span))
        }
    }

    fn write_bytes(&mut self, bytes: &[u8], show_ascii: bool) -> fmt::Result {
        let mut iter = bytes.iter().copied();
        if let Some(byte) = iter.next() {
            self.write_byte(byte, bytes, show_ascii)?;
        }
        for (i, byte) in (1..bytes.len()).zip(iter) {
            self.write_space(1)?;
            self.write_byte(byte, &bytes[i..], show_ascii)?;
        }
        Ok(())
    }

    fn write_byte(&mut self, byte: u8, remaining: &[u8], show_ascii: bool) -> fmt::Result {
        if self.underline {
            let byte_display_width = byte_display_width(byte, show_ascii);
            if is_section_start_within_span(remaining, self.span) {
                self.write_underline(byte_display_width)
            } else {
                self.write_space(byte_display_width)
            }
        } else {
            byte_display_write(byte, show_ascii, &mut self.w)
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Str

    pub(super) fn write_str_side(&mut self, side: &str, cjk: bool) -> fmt::Result {
        self.write_str_open(side)?;
        self.write_str(side, cjk)?;
        self.write_str_close(side)
    }

    pub(super) fn write_str_sides(&mut self, left: &str, right: &str, cjk: bool) -> fmt::Result {
        self.write_str_open(left)?;
        self.write_str(left, cjk)?;
        self.write_delim('"', false)?;
        self.write_space(1)?;
        self.write_more(is_span_overlapping_end(left.as_bytes(), self.span))?;
        self.write_space(1)?;
        self.write_delim('"', false)?;
        self.write_str(right, cjk)?;
        self.write_str_close(right)
    }

    fn write_str_open(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if has_more_before(bytes, self.full) {
            self.write_more(is_span_overlapping_start(bytes, self.span))?;
            self.write_space(1)?;
            self.write_delim('"', false)
        } else {
            self.write_delim('"', is_span_pointing_to_start(bytes, self.span))
        }
    }

    fn write_str_close(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if has_more_after(bytes, self.full) {
            self.write_delim('"', false)?;
            self.write_space(1)?;
            self.write_more(is_span_overlapping_end(bytes, self.span))
        } else {
            self.write_delim('"', is_span_pointing_to_end(bytes, self.span))
        }
    }

    fn write_str(&mut self, s: &str, cjk: bool) -> fmt::Result {
        let bytes = s.as_bytes();
        if self.underline {
            if is_span_start_within_section(bytes, self.span) {
                let mut offset = 0;
                for c in s.chars() {
                    let char_display_width = char_display_width(c, cjk);
                    if is_section_start_within_span(&bytes[offset..], self.span) {
                        self.write_underline(char_display_width)?;
                    } else {
                        self.write_space(char_display_width)?;
                    }
                    offset += c.len_utf8();
                }
            } else {
                for c in s.chars() {
                    self.write_space(char_display_width(c, cjk))?;
                }
            }
        } else {
            for c in s.chars() {
                char_display_write(c, &mut self.w)?;
            }
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    // Private

    fn write_more(&mut self, highlight: bool) -> fmt::Result {
        if self.underline {
            if highlight {
                self.write_underline(2)
            } else {
                self.write_space(2)
            }
        } else {
            self.w.write_str("..")
        }
    }

    fn write_delim(&mut self, delim: char, highlighted: bool) -> fmt::Result {
        if self.underline {
            if highlighted {
                self.write_underline(1)
            } else {
                self.write_space(1)
            }
        } else {
            self.w.write_char(delim)
        }
    }

    fn write_space(&mut self, len: usize) -> fmt::Result {
        self.write_char_len(' ', len)
    }

    fn write_underline(&mut self, len: usize) -> fmt::Result {
        self.write_char_len('^', len)
    }

    fn write_char_len(&mut self, c: char, len: usize) -> fmt::Result {
        for _ in 0..len {
            self.w.write_char(c)?;
        }
        Ok(())
    }
}

fn has_more_before(bytes: &[u8], full: &[u8]) -> bool {
    let section_bounds = bytes.as_ptr_range();
    let full_bounds = full.as_ptr_range();
    section_bounds.start > full_bounds.start
}

fn has_more_after(bytes: &[u8], full: &[u8]) -> bool {
    let section_bounds = bytes.as_ptr_range();
    let full_bounds = full.as_ptr_range();
    section_bounds.end < full_bounds.end
}

fn is_span_start_within_section(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start <= span_bounds.start && section_bounds.end > span_bounds.start
    })
}

fn is_section_start_within_span(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start >= span_bounds.start && section_bounds.start < span_bounds.end
    })
}

fn is_span_overlapping_end(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.end < span_bounds.end
    })
}

fn is_span_overlapping_start(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        section_bounds.start > span_bounds.start
    })
}

fn is_span_pointing_to_start(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        span.is_empty() && section_bounds.start == span_bounds.start
    })
}

fn is_span_pointing_to_end(bytes: &[u8], span: Option<&[u8]>) -> bool {
    span.map_or(false, |span| {
        let section_bounds = bytes.as_ptr_range();
        let span_bounds = span.as_ptr_range();
        span.is_empty() && section_bounds.end == span_bounds.end
    })
}
