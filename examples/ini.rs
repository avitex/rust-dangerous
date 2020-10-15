use dangerous::{Error, Expected, Reader};
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input_data = Vec::new();
    io::stdin().read_to_end(&mut input_data)?;
    let input = dangerous::input(input_data.as_ref());
    match input.read_all::<_, _, Expected>(read_ini) {
        Ok(ini) => println!("{:#?}", ini),
        Err(e) => eprintln!("{:#}", e),
    };
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Pair<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
struct Document<'a> {
    globals: Vec<Pair<'a>>,
    sections: Vec<Section<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
struct Section<'a> {
    name: &'a str,
    values: Vec<Pair<'a>>,
}

fn read_ini<'i, E>(r: &mut Reader<'i, E>) -> Result<Document<'i>, E>
where
    E: Error<'i>,
{
    let globals = read_values_until_section(r)?;
    let mut sections = Vec::new();
    while !r.at_end() {
        sections.push(read_section(r)?);
    }
    Ok(Document { globals, sections })
}

fn read_values_until_section<'i, E>(r: &mut Reader<'i, E>) -> Result<Vec<Pair<'i>>, E>
where
    E: Error<'i>,
{
    let mut out = Vec::new();
    fn is_bare_text(c: u8) -> bool {
        !c.is_ascii_whitespace() && c != b'=' && c != b'['
    }
    while !r.at_end() {
        skip_whitespace_or_comment(r);
        let name = r.context("property name", |r| {
            r.take_while(is_bare_text).to_dangerous_non_empty_str()
        })?;
        skip_whitespace_or_comment(r);

        r.consume_u8(b'=')?;

        skip_whitespace_or_comment(r);
        let value = r.context("property value", |r| {
            r.take_while(|c| !c.is_ascii_whitespace() && c != b'=' && c != b'[')
                .to_dangerous_non_empty_str()
        })?;
        skip_whitespace_or_comment(r);
        out.push(Pair { name, value })
    }
    Ok(out)
}

fn read_section<'i, E>(_r: &mut Reader<'i, E>) -> Result<Section<'i>, E>
where
    E: Error<'i>,
{
    unimplemented!("read section")
}

#[cfg(test)]
mod tests {
    use super::*;
    static GLOBALS_WITHOUT_SECTIONS: &[u8] = br#"
            ; comment before
          hello = value
          a = b  ; comment
          ; comment after
        "#;

    #[test]
    fn global_values_with_comments() {
        let values = dangerous::input(GLOBALS_WITHOUT_SECTIONS)
            .read_all::<_, _, Expected>(read_values_until_section)
            .expect("success");
        assert_eq!(
            values,
            vec![
                Pair {
                    name: "hello",
                    value: "value"
                },
                Pair {
                    name: "a",
                    value: "b"
                }
            ]
        )
    }
}

fn skip_whitespace_or_comment<'i, E>(r: &mut Reader<'i, E>)
where
    E: Error<'i>,
{
    let (mut last, mut current) = (0, 0);
    loop {
        current += skip_comment(r);
        current += r.skip_while(|c| c.is_ascii_whitespace());
        if last == current {
            break;
        }
        last = current;
    }
}

fn skip_comment<'i, E>(r: &mut Reader<'i, E>) -> usize
where
    E: Error<'i>,
{
    if r.peek_eq(b";") {
        r.skip_while(|c| c != b'\n')
    } else {
        0
    }
}
