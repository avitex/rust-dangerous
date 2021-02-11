use dangerous::{Expected, Input};

fn main() {
    // Expect length
    dbg!(&dangerous::input(b"A\xC3\xA9 \xC2")
        .to_dangerous_str::<Expected<'_>>()
        .unwrap_err());
    // Expect valid
    dbg!(&dangerous::input(b"A\xC3\xA9 \xFF")
        .to_dangerous_str::<Expected<'_>>()
        .unwrap_err());
    // Expect value
    dbg!(&dangerous::input(b"hello")
        .read_all::<_, _, Expected<'_>>(|r| r.consume(b"world"))
        .unwrap_err());
    // Expecte value: fatal
    dbg!(&dangerous::input(b"hello")
        .into_bound()
        .read_all::<_, _, Expected<'_>>(|r| r.consume(b"world"))
        .unwrap_err());
}
