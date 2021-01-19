#![no_main]
use libfuzzer_sys::fuzz_target;

use core::fmt::{self, Write as _};

use dangerous::{Input, Expected};
use rand::{rngs::SmallRng, Rng, SeedableRng};

macro_rules! read_partial {
    ($input:expr, $read_fn:expr) => {
        let _ = $input.read_partial::<_, _, Expected>($read_fn);
    };
}

fuzz_target!(|data: &[u8]| {
    let mut rng = derive_rng(data);

    let single_slice = &[rng.gen()][..];

    let input_full = dangerous::input(data);

    let (input_a, input_b) = if input_full.is_empty() {
        (input_full.clone(), input_full.clone())
    } else {
        let (a, b) = data.split_at(rng.gen_range(0..data.len()));
        (dangerous::input(a), dangerous::input(b))
    };

    let _ = input_a.is_within(&input_b);

    if let Err(err) = input_full.to_dangerous_str::<Expected>() {
        write!(DummyWrite, "{}", err).unwrap();
    }

    write!(DummyWrite, "{}", input_full.display().full()).unwrap();
    write!(DummyWrite, "{}", input_full.display().head(rng.gen())).unwrap();
    write!(DummyWrite, "{}", input_full.display().tail(rng.gen())).unwrap();
    write!(DummyWrite, "{}", input_full.display().head_tail(rng.gen())).unwrap();
    write!(DummyWrite, "{}", input_full.display().span(&input_a, rng.gen())).unwrap();

    read_partial!(input_full.clone(), |r| r.read_u8());
    read_partial!(input_full.clone(), |r| r.peek_u8());
    read_partial!(input_full.clone(), |r| Ok(r.peek_eq(single_slice)));
    read_partial!(input_full.clone(), |r| r.take(rng.gen()));
    read_partial!(input_full.clone(), |r| r.peek(1).map(drop));
    read_partial!(input_full.clone(), |r| r.skip(rng.gen()));
    read_partial!(input_full.clone(), |r| Ok(r.take_while(|c| c == rng.gen())));
    read_partial!(input_full.clone(), |r| Ok(r.skip_while(|c| c == rng.gen())));
    read_partial!(input_full.clone(), |r| r.take_str_while(|c| c == rng.gen()));
    read_partial!(input_full.clone(), |r| r.skip_str_while(|c| c == rng.gen()));
    read_partial!(input_full.clone(), |r| r.try_take_while(|c| Ok(c == rng.gen())));
    read_partial!(input_full.clone(), |r| r.try_take_str_while(|c| Ok(c == rng.gen())));
    read_partial!(input_full.clone(), |r| r.try_skip_str_while(|c| Ok(c == rng.gen())));
    read_partial!(input_full, |r| r.consume(single_slice));
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
    let mut arr = [0u8; 32];
    for (i, b) in seed.iter().copied().take(32).enumerate() {
        arr[i] = b;
    }
    SmallRng::from_seed(arr)
}
