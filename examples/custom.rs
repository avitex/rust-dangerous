use std::net::Ipv4Addr;

use dangerous::{Error, Expected, ExpectedLength, ExpectedValid, Invalid};

fn main() {
    let input = dangerous::input(b"192.168.1.x");
    let error: Expected = read_ipv4_addr(input).unwrap_err();

    println!("{}", error);
}

fn read_ipv4_addr<'i, E>(input: &'i dangerous::Input) -> Result<Ipv4Addr, E>
where
    E: Error<'i>,
    E: From<ExpectedValid<'i>>,
    E: From<ExpectedLength<'i>>,
{
    input.read_all_erased("ipv4 addr", |i| {
        i.take_remaining()
            .to_dangerous_str()
            .and_then(|s| s.parse().map_err(|_| Invalid::default()))
    })
}
