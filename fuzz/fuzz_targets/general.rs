#![no_main]

use dangerous::{Expected, Input};
use libfuzzer_sys::fuzz_target;

macro_rules! read_partial {
    ($input:expr, $read_fn:expr) => {
        let _ = $input.read_partial::<_, _, Expected>($read_fn);
    };
}

fuzz_target!(|data_length_and_patterns: (&[u8], usize, u8, &[u8])| {
    let (data, len, byte_pattern, slice_pattern) = data_length_and_patterns;
    let input = dangerous::input(data);

    // read
    read_partial!(input.clone(), |r| r.read());
    // peek
    read_partial!(input.clone(), |r| r.peek_read());
    read_partial!(input.clone(), |r| r.peek(len).map(drop));
    read_partial!(input.clone(), |r| Ok(r.peek_eq(slice_pattern)));
    // consume
    read_partial!(input.clone(), |r| r.consume(byte_pattern));
    read_partial!(input.clone(), |r| r.consume(slice_pattern));
    // take/skip
    read_partial!(input.clone(), |r| r.take(len));
    read_partial!(input.clone(), |r| r.skip(len));
    // (take/skip/try_take/try_skip)_while
    read_partial!(input.clone(), |r| Ok(r.skip_while(byte_pattern)));
    read_partial!(input.clone(), |r| Ok(r.skip_while(slice_pattern)));
    read_partial!(input.clone(), |r| Ok(r.take_while(|c| c == byte_pattern)));
    read_partial!(input.clone(), |r| Ok(r.skip_while(|c| c == byte_pattern)));
    read_partial!(input.clone(), |r| r
        .try_take_while(|c| Ok(c == byte_pattern)));
    read_partial!(input.clone(), |r| r
        .try_skip_while(|c| Ok(c == byte_pattern)));
    // (take/skip)_until
    read_partial!(input.clone(), |r| Ok(r.take_until(byte_pattern)));
    read_partial!(input.clone(), |r| Ok(r.take_until(slice_pattern)));
    read_partial!(input.clone(), |r| Ok(
        r.take_until_opt(|c| c == byte_pattern)
    ));
    read_partial!(input.clone(), |r| Ok(
        r.take_until_opt(|c| c == byte_pattern)
    ));
});
