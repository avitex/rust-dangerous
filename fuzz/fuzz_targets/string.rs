#![no_main]

use dangerous::{Expected, Input};
use libfuzzer_sys::fuzz_target;

macro_rules! read_partial {
    ($input:expr, $read_fn:expr) => {
        let _ = $input.read_partial::<_, _, Expected>($read_fn);
    };
}

fuzz_target!(|data_and_delim: (&[u8], char)| {
    let (data, delim) = data_and_delim;
    let input = dangerous::input(data);

    read_partial!(input.clone(), |r| r.take_str_while(|c| c == delim));
    read_partial!(input.clone(), |r| r.skip_str_while(|c| c == delim));
    read_partial!(input.clone(), |r| r.try_take_str_while(|c| Ok(c == delim)));
    read_partial!(input.clone(), |r| r.try_skip_str_while(|c| Ok(c == delim)));
});
