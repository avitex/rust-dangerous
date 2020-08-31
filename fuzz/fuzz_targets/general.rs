#![no_main]
use libfuzzer_sys::fuzz_target;

use core::fmt::{self, Write as _};

use dangerous::{Invalid, Expected};

struct DummyWrite;

impl fmt::Write for DummyWrite {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Ok(())
    }
}

fuzz_target!(|data: &[u8]| {
    let _ = dangerous::input(data).to_dangerous_str::<Invalid>();
    
    if let Err(err) = dangerous::input(data).to_dangerous_str::<Expected>() {
        write!(DummyWrite, "{}", err).unwrap();
    }

    // TODO: MORE!
});
