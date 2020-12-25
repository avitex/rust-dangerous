use dangerous::{Expected, Reader};
use zc::Dependant;

#[derive(Dependant, Debug)]
pub struct ParsedResult<'a>(Result<Vec<&'a str>, Expected<'a>>);

impl<'a> From<&'a [u8]> for ParsedResult<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        let input = dangerous::input(bytes);
        Self(input.read_all(parse))
    }
}

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);
    let parsed = zc::from!(buf, ParsedResult, [u8]);
    dbg!(parsed);
}

fn parse<'i, E>(r: &mut Reader<'i, E>) -> Result<Vec<&'i str>, E>
where
    E: dangerous::Error<'i>,
{
    let mut parts = Vec::new();
    loop {
        let s = r.take_while(|b| b != b',').to_dangerous_str::<E>()?;
        parts.push(s);
        if !r.consume_u8_opt(b',') {
            return Ok(parts);
        }
    }
}
