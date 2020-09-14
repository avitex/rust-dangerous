use dangerous::Expected;

fn main() {
    let input = dangerous::input(b"hello<123>");

    let err = input
        .read_all::<_, _, Expected>(|r| {
            r.context("protocol", |r| {
                let _ = r.take_while(|b| b.is_ascii_alphabetic());
                r.consume(b"<")?;
                r.context("number", |r| {
                    let number = r.take_while(|b| b != b'>');
                    number.read_all(|r| r.consume(b"124"))
                })
            })
        })
        .unwrap_err();

    println!("{}", err.display());
}
