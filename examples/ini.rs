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

fn read_values_until_section<'i, E>(_r: &mut Reader<'i, E>) -> Result<Vec<Pair<'i>>, E>
where
    E: Error<'i>,
{
    unimplemented!("read values")
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

    #[test]
    fn global_value() {
        let input = dangerous::input(
            br#"
          hello = value
        "#,
        );
        let ini = input.read_all::<_, _, Expected>(read_ini).expect("success");
        assert_eq!(
            ini,
            Document {
                globals: vec![Pair {
                    name: "hello",
                    value: "value"
                }],
                sections: vec![],
            }
        )
    }
}

fn skip_whitespace<E>(r: &mut Reader<'_, E>) {
    r.skip_while(|c| c.is_ascii_whitespace());
}
