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

mod verbose {
    use nom::{
        bytes::streaming::tag, character::streaming::char, error::context, error::VerboseError,
        IResult,
    };

    pub fn parse(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
        context("a", |i| {
            context("b", |i| {
                let (i, _) = char('f')(i)?;
                tag("oobar")(i)
            })(i)
        })(i)
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

fn parse_verbose<'i, E>(r: &mut StringReader<'i, E>) -> Result<&'i str, E>
where
    E: Error<'i>,
{
    r.take_remaining().read_all(|r| {
        r.try_expect_external("value", |i| {
            verbose::parse(i.as_dangerous())
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
fn test_parse_verbose_ok() {
    let value = read_all_ok!("foobar", parse_verbose);
    assert_eq!(value, "oobar");
}

#[test]
fn test_parse_color_too_short() {
    let error = read_all_err!("#2F14D", parse_color);
    assert!(error.is_fatal());
    assert_str_eq!(
        format!("{:#}\n", error),
        indoc! {r##"
            error attempting to read and expect an external value: expected hex color
            > "#2F14D"
                    ^ 
            additional:
              error line: 1, error offset: 5, input length: 6
            backtrace:
              1. `read all input`
              2. `read all input`
              3. `read and expect an external value` (expected hex color)
                1. `TakeWhileMN`
        "##}
    );
}

#[test]
fn test_parse_verbose_err() {
    let error = read_all_err!("err", parse_verbose);
    assert!(error.is_fatal());
    assert_str_eq!(
        format!("{:#}\n", error),
        indoc! {r##"
            error attempting to read and expect an external value: expected value
            > "err"
               ^^^ 
            additional:
              error line: 1, error offset: 0, input length: 3
            backtrace:
              1. `read all input`
              2. `read all input`
              3. `read and expect an external value` (expected value)
                1. `<context>` (expected a)
                2. `<context>` (expected b)
                3. `consume input` (expected character 'f')
        "##}
    );
}

#[test]
fn test_parse_verbose_err_retry() {
    let error = read_all_err!("f", parse_verbose);
    assert!(!error.is_fatal());
    assert_eq!(error.to_retry_requirement(), RetryRequirement::new(5));
    assert_str_eq!(
        format!("{:#}\n", error),
        indoc! {r##"
            error attempting to read and expect an external value: expected value
            > "f"
               ^ 
            additional:
              error line: 1, error offset: 0, input length: 1
            backtrace:
              1. `read all input`
              2. `read all input`
              3. `read and expect an external value` (expected value)
        "##}
    );
}
