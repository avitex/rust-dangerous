#[macro_use]
mod common;

use common::*;

///////////////////////////////////////////////////////////////////////////////

mod color {
    use nom::{
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        sequence::tuple,
        IResult,
    };

    #[derive(Debug, PartialEq)]
    pub struct Value {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
    }

    pub fn parse(input: &str) -> IResult<&str, Value> {
        let (input, _) = tag("#")(input)?;
        let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

        Ok((input, Value { red, green, blue }))
    }

    fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn hex_primary(input: &str) -> IResult<&str, u8> {
        map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
    }
}

///////////////////////////////////////////////////////////////////////////////

fn parse_color<'i, E>(r: &mut StringReader<'i, E>) -> Result<color::Value, E>
where
    E: Error<'i>,
{
    r.take_remaining().read_all(|r| {
        r.try_expect_external("hex color", |i| {
            color::parse(i.as_dangerous())
                .map(|(remaining, response)| (response, i.byte_len() - remaining.len()))
        })
    })
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_parse_color_ok() {
    assert_eq!(
        read_all_ok!("#2F14DF", parse_color),
        color::Value {
            red: 47,
            green: 20,
            blue: 223,
        }
    );
}

#[test]
fn test_parse_color_trailing() {
    let _ = read_all_err!("#2F14DFF", parse_color);
}

#[test]
fn test_parse_color_too_short() {
    let error = read_all_err!("#2F14D", parse_color);
    assert_str_eq!(
        format!("{:#}\n", error),
        indoc! {r##"
            error attempting to try expect external: expected hex color
            > "#2F14D"
                    ^ 
            additional:
              error line: 1, error offset: 5, input length: 6
            backtrace:
              1. `read all`
              2. `read all`
              3. `try expect external` (expected hex color)
                1. `TakeWhileMN`
        "##}
    );
}
