use dangerous::{BytesReader, Error, Expected, Input};
use zc::Zc;

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);
    let result = Zc::new(buf, parse_bytes);
    dbg!(&result);
}

fn parse_bytes<'i>(bytes: &'i [u8]) -> Result<Vec<&'i str>, Expected<'i>> {
    dangerous::input(bytes).read_all(parse)
}

fn parse<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Vec<&'i str>, E>
where
    E: Error<'i>,
{
    let mut parts = Vec::new();
    loop {
        let s = r.take_until_opt(b',').to_dangerous_str::<E>()?;
        parts.push(s);
        if !r.consume_opt(b',') {
            return Ok(parts);
        }
    }
}
