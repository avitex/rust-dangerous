use core::convert::TryFrom;
use dangerous::{BytesReader, Expected, Input};
use zc::Dependant;

#[derive(Dependant, Debug)]
pub struct ParsedResult<'a>(Vec<&'a str>);

impl<'a> TryFrom<&'a [u8]> for ParsedResult<'a> {
    type Error = String;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        dangerous::input(bytes)
            .read_all(parse)
            .map(Self)
            .map_err(|err: Expected| err.to_string())
    }
}

fn main() {
    let buf = Vec::from(&b"thisisatag,thisisanothertag"[..]);
    let result = zc::try_from!(buf, ParsedResult, [u8]);
    dbg!(&result);
}

fn parse<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Vec<&'i str>, E>
where
    E: dangerous::Error<'i>,
{
    let mut parts = Vec::new();
    loop {
        let s = r.take_while(|b| b != b',').to_dangerous_str::<E>()?;
        parts.push(s);
        if !r.consume_opt(b',') {
            return Ok(parts);
        }
    }
}
