#![no_main]
use libfuzzer_sys::fuzz_target;

use core::fmt::{self, Write as _};

use rand::{Rng, SeedableRng, rngs::SmallRng};
use dangerous::Expected;

fuzz_target!(|data: &[u8]| {
    let mut rng = derive_rng(data);

    let single_slice =  &[rng.gen()][..];

    let input_full = dangerous::input(data);

    let (input_a, input_b) = if data.is_empty() {
        (input_full, input_full)
    } else {
        let (a, b) = data.split_at(rng.gen_range(0, data.len()));
        (dangerous::input(a), dangerous::input(b))
    };

    let _ = input_a.is_within(input_b);
    
    if let Err(err) = input_full.to_dangerous_str::<Expected>() {
        write!(DummyWrite, "{}", err).unwrap();
    }

    let _ = input_full.reader::<Expected>().read_u8();
    let _ = input_full.reader::<Expected>().peek_u8();
    let _ = input_full.reader::<Expected>().peek_eq(single_slice);
    let _ = input_full.reader::<Expected>().take(rng.gen());
    let _ = input_full.reader::<Expected>().skip(rng.gen());
    let _ = input_full.reader::<Expected>().take_while(|_, c| c == rng.gen());
    let _ = input_full.reader::<Expected>().try_take_while(|_, c| Ok(c == rng.gen()));
    let _ = input_full.reader::<Expected>().peek(1, |i| i == single_slice);
    let _ = input_full.reader::<Expected>().try_peek(1, |i| Ok(i == single_slice));
    let _ = input_full.reader::<Expected>().consume(single_slice);
});


///////////////////////////////////////////////////////////////////////////////
// Support

struct DummyWrite;

impl fmt::Write for DummyWrite {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Ok(())
    }
}

fn derive_rng(seed: &[u8]) -> SmallRng {
    let mut arr = [0u8; 16];
    for (i, b) in seed.iter().copied().take(16).enumerate() {
        arr[i] = b;
    }
    SmallRng::from_seed(arr)
}
