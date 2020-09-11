use dangerous::Expected;

fn main() {
    let input = dangerous::input(b"hello<123>");

    let err = input
        .read_all::<_, _, Expected>(|r| {
            r.context("read protocol", |r| {
                let _ = r.take_while(|_, b| b.is_ascii_alphabetic());
                r.consume(b"<")?;
                let number = r.take_while(|_, b| b != b'>');
                number.read_all(|r| r.consume(b"124"))
            })
        })
        .unwrap_err();

    println!("{}", err.display());
}
