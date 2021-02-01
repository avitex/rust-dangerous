use std::net::Ipv4Addr;

use dangerous::{BytesReader, Error, Expected, Input, Invalid};

fn main() {
    let input = dangerous::input(b"192.168.1.x");
    let error: Expected<'_> = input.read_all(read_ipv4_addr).unwrap_err();

    println!("{:#}", error);
}

fn read_ipv4_addr<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Ipv4Addr, E>
where
    E: Error<'i>,
{
    r.try_expect_erased("ipv4 addr", |r| {
        r.take_remaining()
            .to_dangerous_str::<Invalid>()?
            .parse()
            .map_err(|_| Invalid::fatal())
    })
}
