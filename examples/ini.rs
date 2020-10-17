//! A basic rather tolerant ini parser, which most certainly does not obey to the standard
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

#[derive(Default, Debug, PartialEq, Eq)]
struct Document<'a> {
    globals: Vec<Pair<'a>>,
    sections: Vec<Section<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
struct Section<'a> {
    name: &'a str,
    properties: Vec<Pair<'a>>,
}

fn read_ini<'i, E>(r: &mut Reader<'i, E>) -> Result<Document<'i>, E>
where
    E: Error<'i>,
{
    skip_whitespace_or_comment(r, ConsumeTo::NextToken);
    Ok(match r.peek_u8().ok() {
        None => Document::default(),
        Some(b'[') => Document {
            globals: vec![],
            sections: read_sections(r)?,
        },
        Some(_) => Document {
            globals: read_zero_or_more_properties_until_section(r)?,
            sections: read_sections(r)?,
        },
    })
}

fn read_sections<'i, E>(r: &mut Reader<'i, E>) -> Result<Vec<Section<'i>>, E>
where
    E: Error<'i>,
{
    let mut sections = Vec::new();
    while !r.at_end() {
        sections.push(read_section(r)?);
    }
    Ok(sections)
}

fn read_zero_or_more_properties_until_section<'i, E>(
    r: &mut Reader<'i, E>,
) -> Result<Vec<Pair<'i>>, E>
where
    E: Error<'i>,
{
    let mut out = Vec::new();
    fn is_bare_text(c: u8) -> bool {
        !(c.is_ascii_whitespace() || c == b'=' || c == b'[')
    }

    skip_whitespace_or_comment(r, ConsumeTo::NextToken);
    while !(r.at_end() || r.peek_eq(b"[")) {
        r.context("property", |r| {
            skip_whitespace_or_comment(r, ConsumeTo::NextToken);
            let name = r.context("name", |r| {
                r.take_while(is_bare_text).to_dangerous_non_empty_str()
            })?;
            skip_whitespace_or_comment(r, ConsumeTo::EndOfLine);

            r.consume_u8(b'=')?;

            skip_whitespace_or_comment(r, ConsumeTo::EndOfLine);
            let value = r.context("value", |r| {
                r.take_while(|c| c != b';' && c != b'\n' && c != b'=' && c != b'[')
                    .to_dangerous_non_empty_str()
                    .map(str::trim)
            })?;
            skip_whitespace_or_comment(r, ConsumeTo::NextToken);
            out.push(Pair { name, value });
            Ok(())
        })?;
    }
    Ok(out)
}

fn read_section<'i, E>(r: &mut Reader<'i, E>) -> Result<Section<'i>, E>
where
    E: Error<'i>,
{
    skip_whitespace_or_comment(r, ConsumeTo::NextToken);
    r.consume_u8(b'[')?;
    let name = r.context("section name", |r| {
        r.take_while(|c| c != b']' && c != b'\n')
            .to_dangerous_non_empty_str()
            .map(str::trim)
    })?;
    r.consume_u8(b']')?;

    r.verify("newline after section", |r| {
        r.take_while(|c| c.is_ascii_whitespace())
            .as_dangerous()
            .contains(&b'\n')
    })?;

    let properties = read_zero_or_more_properties_until_section(r)?;
    Ok(Section { name, properties })
}

enum ConsumeTo {
    NextToken,
    EndOfLine,
}

fn skip_whitespace_or_comment<E>(r: &mut Reader<'_, E>, to_where: ConsumeTo) {
    fn skip_comment<E>(r: &mut Reader<E>) -> usize {
        if r.peek_eq(b";") {
            r.skip_while(|c| c != b'\n')
        } else {
            0
        }
    }

    let (mut last, mut current) = (0, 0);
    loop {
        current += skip_comment(r);
        current += r.skip_while(|c| {
            let iwb = c.is_ascii_whitespace();
            iwb && match to_where {
                ConsumeTo::NextToken => true,
                ConsumeTo::EndOfLine => c != b'\n',
            }
        });
        if last == current {
            break;
        }
        last = current;
    }
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

    static SECTION_WITHOUT_VALUES: &[u8] = br#"
            ; comment before
            [ section name ]
          ; comment after
        "#;

    static INI: &[u8] = br#"language=rust ; awesome

[ section ]
name = dangerous ;
type = manual

[empty section]
"#;

    #[test]
    fn section_without_values() {
        let section = dangerous::input(SECTION_WITHOUT_VALUES)
            .read_all::<_, _, Expected>(read_section)
            .expect("success");
        assert_eq!(
            section,
            Section {
                name: "section name",
                properties: vec![]
            },
        )
    }

    #[test]
    fn complete_ini_file() {
        let ini = dangerous::input(INI)
            .read_all::<_, _, Expected>(read_ini)
            .expect("success");
        assert_eq!(
            ini,
            Document {
                globals: vec![Pair {
                    name: "language",
                    value: "rust"
                }],
                sections: vec![
                    Section {
                        name: "section",
                        properties: vec![
                            Pair {
                                name: "name",
                                value: "dangerous"
                            },
                            Pair {
                                name: "type",
                                value: "manual"
                            }
                        ]
                    },
                    Section {
                        name: "empty section",
                        properties: vec![]
                    }
                ]
            },
        )
    }

    #[test]
    fn global_values_with_comments() {
        let values = dangerous::input(GLOBALS_WITHOUT_SECTIONS)
            .read_all::<_, _, Expected>(read_zero_or_more_properties_until_section)
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

    #[test]
    fn document_without_sections() {
        let ini = dangerous::input(GLOBALS_WITHOUT_SECTIONS)
            .read_all::<_, _, Expected>(read_ini)
            .expect("success");
        assert_eq!(
            ini,
            Document {
                globals: vec![
                    Pair {
                        name: "hello",
                        value: "value"
                    },
                    Pair {
                        name: "a",
                        value: "b"
                    }
                ],
                sections: vec![]
            }
        )
    }

    #[test]
    fn empty_input() {
        let ini = dangerous::input(b"")
            .read_all::<_, _, Expected>(read_ini)
            .expect("success");
        assert_eq!(ini, Document::default());

        let ini = dangerous::input(b"  \n ; empty ")
            .read_all::<_, _, Expected>(read_ini)
            .expect("success");
        assert_eq!(ini, Document::default())
    }
}
