use imap_proto::{parser, types};

use dangerous::{BytesReader, Error, Expected, Input};

fn main() {
    println!("=== VALID PARSE ===");
    let input = dangerous::input(&b"* LIST (\\HasNoChildren) \".\" INBOX.Tests\r\n"[..]);
    let result: Result<_, Expected<'_>> = input.read_all(read_imap_response);
    println!("{:?}", result.unwrap());

    println!("\n=== INVALID PARSE ===");
    let input = dangerous::input(&b"* LIST (\\HasNoChildren) \".\" IN\"BOX.Tests\r\n"[..]);
    let error: Expected<'_> = input.read_all(read_imap_response).unwrap_err();
    println!("{:#}", error);

    println!("\n=== INVALID PARSE: TRAILING INPUT ===");
    let input =
        dangerous::input(&b"* LIST (\\HasNoChildren) \".\" INBOX.Tests\r\ni am trailing"[..]);
    let error: Expected<'_> = input.read_all(read_imap_response).unwrap_err();
    println!("{:#}", error);
}

fn read_imap_response<'i, E>(r: &mut BytesReader<'i, E>) -> Result<types::Response<'i>, E>
where
    E: Error<'i>,
{
    r.try_external("IMAP response", |i| {
        parser::parse_response(i.as_dangerous())
            .map(|(remaining, response)| (i.len() - remaining.len(), response))
    })
}
