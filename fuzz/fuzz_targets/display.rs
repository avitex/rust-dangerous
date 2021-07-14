#![no_main]

use dangerous::{Expected, Input};
use libfuzzer_sys::fuzz_target;
use std::fmt::{self, Write};

fuzz_target!(|data_len_and_width: (&[u8], usize, usize)| {
    let (data, len, width) = data_len_and_width;
    let input = dangerous::input(data);

    if let Err(err) = input.to_dangerous_str::<Expected>() {
        write!(DummyWrite, "{}", err).unwrap();
    }

    write!(DummyWrite, "{}", input.display().full()).unwrap();
    write!(DummyWrite, "{}", input.display().head(width)).unwrap();
    write!(DummyWrite, "{}", input.display().head(width)).unwrap();
    write!(DummyWrite, "{}", input.display().tail(width)).unwrap();
    write!(DummyWrite, "{}", input.display().head_tail(width)).unwrap();

    if let (Some(input_span), _) = input.clone().read_infallible(|r| r.take_opt(len)) {
        write!(
            DummyWrite,
            "{}",
            input.display().span(input_span.span(), width)
        )
        .unwrap();
    }
});

struct DummyWrite;

impl fmt::Write for DummyWrite {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Ok(())
    }
}
