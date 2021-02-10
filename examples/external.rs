use core::convert::TryFrom;

use dangerous::{error, BytesReader, Error, Expected, Input};

fn main() {
    println!("=== VALID PARSE ===");
    let input = dangerous::input(b"foo");
    let result: Result<_, Expected<'_>> = input.read_all(read_custom);
    println!("{:?}", result.unwrap());

    println!("\n=== INVALID PARSE ===");
    let input = dangerous::input(b"bar");
    let error: Expected<'_> = input.read_all(read_custom).unwrap_err();
    println!("{:#}", error);
}

#[derive(Debug)]
pub struct Custom<'i>(&'i str);

impl<'i> TryFrom<dangerous::String<'i>> for Custom<'i> {
    type Error = ParseCustomError;

    fn try_from(s: dangerous::String<'i>) -> Result<Self, Self::Error> {
        if s.as_dangerous() == "foo" {
            Ok(Self(s.as_dangerous()))
        } else {
            Err(ParseCustomError)
        }
    }
}

pub struct ParseCustomError;

impl<'i> error::External<'i> for ParseCustomError {
    fn expected(&self) -> Option<&'static str> {
        Some("my custom type")
    }
}

fn read_custom<'i, E>(r: &mut BytesReader<'i, E>) -> Result<Custom<'i>, E>
where
    E: Error<'i>,
{
    r.take_remaining_str()?
        .into_external("my custom type", Custom::try_from)
}
