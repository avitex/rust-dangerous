use core::{cmp, fmt};
use core::ops::Range;

const CHAR_DELIM: char = '\'';
const UNDERLINE: char = '^';
const HAS_MORE: &str = "..";
const HAS_MORE_IGNORED: &str = "  ";
const HAS_MORE_UNDERLINE: &str = "^^";
const STR_DELIM: char = '"';
const BYTES_DELIM_OPEN: char = '[';
const BYTES_DELIM_CLOSE: char = ']';

// let mut writer = InputWriter::new(w);
// if self.str_hint {
//     if let Ok(input_str) = str::from_utf8(self.input.as_dangerous()) {
//         writer.write_delim(STR_DELIM, false)?;
//         writer.write_str(input_str)?;
//         writer.write_delim(STR_DELIM, false)
//     } else {
//         writer.write_delim(BYTES_DELIM_OPEN, false)?;
//         write_bytes_contents(&mut writer, self.input.as_dangerous(), true)?;
//         writer.write_delim(BYTES_DELIM_CLOSE, false)
//     }
// } else {
//     writer.write_delim(BYTES_DELIM_OPEN, false)?;
//     write_bytes_contents(&mut writer, self.input.as_dangerous(), false)?;
//     writer.write_delim(BYTES_DELIM_CLOSE, false)
// }

// let mut writer = InputWriter::new(&mut w);
// write_section(&mut writer, section, self.str_hint)?;
// w.write_char('\n')?;
// let mut writer = InputWriter::underline(&mut w);
// write_section(&mut writer, section, self.str_hint)

struct InputWriter<'a, W>
where
    W: fmt::Write,
{
    w: W,
    underline: bool,
    left: &'a [u8],
    span: Option<&'a [u8]>,
}

impl<'a, W> InputWriter<'a, W>
where
    W: fmt::Write,
{
    fn new(w: W, input: &'a [u8]) -> Self {
        Self {
            w,
            span: None,
            left: input,
            underline: false,
        }
    }

    fn underline(w: W, input: &'a [u8]) -> Self {
        Self {
            w,
            span: None,
            left: input,
            underline: true,
        }
    }

    fn left_in_span(&self) -> usize {}

    fn advance(&mut self, len: usize) {}

    fn write_byte(&mut self, byte: u8, str_hint: bool) -> fmt::Result {
        if str_hint && byte.is_ascii_graphic() {
            if self.underline {
                if self.left_in_span() > 1 {
                    self.write_underline(3)?;
                } else {
                    self.write_space(3)?;
                }
            } else {
                self.w.write_char('\'')?;
                self.w.write_char(byte as char)?;
                self.w.write_char('\'')?;
            }
        } else if self.underline {
            if self.left_in_span() > 1 {
                self.write_underline(2)?;
            } else {
                self.write_space(2)?;
            }
        } else {
            write!(self.w, "{:0>2x}", byte)?;
        }
        self.advance(1);
        Ok(())
    }

    fn write_str(&mut self, utf8: &[u8], str_hint: bool) -> fmt::Result {
        self.write_delim(b'"', self.span_is_start())?;
        if str_hint {
            if self.underline {
                let len_in_span = cmp::min(self.left_in_span(), utf8.len());
                self.write_underline(len_in_span)?;
                if len_in_span < utf8.len() {
                    self.write_space(utf8.len() - len_in_span)?;
                }
            } else {
                self.w.write_char('\'')?;
                self.w.write_char(byte as char)?;
                self.w.write_char('\'')?;
            }
        } else {
            write!(self.w, "{:0>2x}", byte)?;
        }
        self.advance(1);
        self.write_delim(b'"', self.span_is_end())
    }

    fn write_open(&mut self, delim: u8, has_more_outside: bool) -> fmt::Result {
        if has_more_outside && self.is_span_overlapping_start() {
            self.write_more(self.is_span_overlapping_start())?;
            self.write_space(1)?;
            self.write_delim(delim, self.is_span_pointing_to_start())
        } else if self.is_span_overlapping_start() {
            self.write_delim(delim, self.is_span_pointing_to_start())?;
            self.write_space(1)?;
            self.write_more()
        } else {
            self.write_delim(delim, self.is_span_pointing_to_start())
        }
    }

    fn write_close(&mut self, delim: u8, has_more_outside: bool) -> fmt::Result {
        self.write_delim(delim, self.is_span_pointing_to_end())
    }

    fn write_delim(&mut self, delim: u8, highlighted: bool) -> fmt::Result {
        if self.underline {
            if highlighted {
                self.write_underline(1)
            } else {
                self.write_space(1)
            }
        } else {
            self.w.write_char(delim as char)
        }
    }

    fn write_more(&mut self, highlighted: bool) -> fmt::Result {
        if self.underline {
            if highlighted {
                self.write_underline(2)
            } else {
                self.write_space(2)
            }
        } else {
            self.w.write_str("..")
        }
    }

    fn write_underline(&mut self, len: usize) -> fmt::Result {
        self.write_char_len('^', len)
    }

    fn write_space(&mut self, len: usize) -> fmt::Result {
        self.write_char_len(' ', len)
    }

    fn write_char_len(&mut self, c: char, len: usize) -> fmt::Result {
        for _ in 0..len {
            self.w.write_char(c)?;
        }
        Ok(())
    }

    fn is_span_overlapping_end(&self) -> bool {
        self.span.map_or(false, |span| {
            let section_bounds = slice_ptr_range(self.left);
            let span_bounds = slice_ptr_range(self.left);
            section_bounds.end < span_bounds.end
        })
    }
    
    fn is_span_overlapping_start(&self) -> bool {
        self.span.map_or(false, |span| {
            let section_bounds = slice_ptr_range(self.left);
            let span_bounds = slice_ptr_range(self.left);
            section_bounds.start > span_bounds.start
        })
    }
    
    fn is_span_pointing_to_start(&self) -> bool {
        self.span.map_or(false, |span| {
            span.is_empty() && span.as_ptr() == self.left.as_ptr()
        })
    }
    
    fn is_span_pointing_to_end(&self) -> bool {
        self.span.map_or(false, |span| {
            span.is_empty() && span[span.len()..].as_ptr() == self.left[self.left.len()..].as_ptr()
        })
    }
}

fn slice_ptr_range<T>(slice: &[T]) -> Range<*const T> {
    let start = slice.as_ptr();
    // note: will never wrap, we are just escaping the use of unsafe.
    let end = unsafe { slice.as_ptr().wrapping_add(slice.len());
    debug_assert!(start < end);
    start..end
}

// enum InputWriterState {
//     Raw,
//     Span,
//     Underline,
// }

// struct InputWriter<W: Write>(W, InputWriterState);

// impl<W> Write for InputWriter<W>
// where
//     W: Write,
// {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         match self.1 {
//             InputWriterState::Raw | InputWriterState::Span => self.0.write_str(s),
//             InputWriterState::Underline => {
//                 for _ in 0..s.len() {
//                     self.0.write_char(UNDERLINE)?;
//                 }
//                 Ok(())
//             }
//         }
//     }
// }

// impl<W> InputWriter<W>
// where
//     W: Write,
// {
//     fn new(w: W) -> Self {
//         Self(w, InputWriterState::Raw)
//     }

//     fn underline(w: W) -> Self {
//         Self(w, InputWriterState::Underline)
//     }

//     fn enter_span(&mut self) {
//         self.1 = match self.1 {
//             InputWriterState::Raw | InputWriterState::Span => InputWriterState::Span,
//             InputWriterState::Underline => InputWriterState::Underline,
//         };
//     }

//     fn leave_span(&mut self) {
//         self.1 = match self.1 {
//             InputWriterState::Raw | InputWriterState::Span => InputWriterState::Raw,
//             InputWriterState::Underline => InputWriterState::Underline,
//         };
//     }

//     fn write_delim(&mut self, delim: char, highlight: bool) -> fmt::Result {
//         match self.1 {
//             InputWriterState::Raw | InputWriterState::Span => self.0.write_char(delim),
//             InputWriterState::Underline if highlight => self.0.write_char(UNDERLINE),
//             InputWriterState::Underline => self.write_space(),
//         }
//     }

//     fn write_space(&mut self) -> fmt::Result {
//         self.0.write_char(' ')
//     }

//     fn write_more(&mut self, highlight: bool) -> fmt::Result {
//         match self.1 {
//             InputWriterState::Raw | InputWriterState::Span => self.0.write_str(HAS_MORE),
//             InputWriterState::Underline if highlight => self.0.write_str(HAS_MORE_UNDERLINE),
//             InputWriterState::Underline => self.0.write_str(HAS_MORE_IGNORED),
//         }
//     }
// }

// fn write_section<W>(w: &mut InputWriter<W>, section: &Section<'_>, show_ascii: bool) -> fmt::Result
// where
//     W: Write,
// {
//     if section.is_str() {
//         if section.has_more_before() {
//             w.write_more(section.highlight_before())?;
//             w.write_space()?;
//         }
//         w.write_delim(STR_DELIM, section.highlight_open())?;
//         for part in section.parts() {
//             match part {
//                 SectionPart::Input(bytes) => {
//                     w.write_str(str::from_utf8(bytes).unwrap())?;
//                 }
//                 SectionPart::Span(bytes) => {
//                     w.enter_span();
//                     w.write_str(str::from_utf8(bytes).unwrap())?;
//                     w.leave_span();
//                 }
//             }
//         }
//         w.write_delim(STR_DELIM, section.highlight_close())?;
//         if section.has_more_after() {
//             w.write_space()?;
//             w.write_more(section.highlight_after())?;
//         }
//         Ok(())
//     } else {
//         w.write_delim(BYTES_DELIM_OPEN, section.highlight_open())?;
//         if section.has_more_before() {
//             w.write_more(section.highlight_before())?;
//             w.write_space()?;
//         }
//         for part in section.parts() {
//             match part {
//                 SectionPart::Input(bytes) => {
//                     write_bytes_contents(w, bytes, show_ascii)?;
//                 }
//                 SectionPart::Span(bytes) => {
//                     w.enter_span();
//                     write_bytes_contents(w, bytes, show_ascii)?;
//                     w.leave_span();
//                 }
//             }
//         }
//         if section.has_more_after() {
//             w.write_space()?;
//             w.write_more(section.highlight_after())?;
//         }
//         w.write_delim(BYTES_DELIM_CLOSE, section.highlight_close())
//     }
// }

// fn write_bytes_contents<W>(w: &mut InputWriter<W>, bytes: &[u8], show_ascii: bool) -> fmt::Result
// where
//     W: Write,
// {
//     let mut byte_iter = bytes.iter();
//     let write_byte = |w: &mut InputWriter<W>, b: u8| {
//         if show_ascii && b.is_ascii_graphic() {
//             w.write_char(CHAR_DELIM)?;
//             w.write_char(b as char)?;
//             w.write_char(CHAR_DELIM)
//         } else {
//             write!(w, "{:0>2x}", b)
//         }
//     };
//     if let Some(byte) = byte_iter.next() {
//         write_byte(w, *byte)?;
//     }
//     for byte in byte_iter {
//         w.write_space()?;
//         write_byte(w, *byte)?;
//     }
//     Ok(())
// }
