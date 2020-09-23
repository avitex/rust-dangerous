use dangerous::Expected;

fn main() {
    // expect length
    dbg!(dangerous::input(b"A\xC3\xA9 \xC2")
        .to_dangerous_str::<Expected>()
        .unwrap_err());
    // expect valid
    dbg!(dangerous::input(b"A\xC3\xA9 \xFF")
        .to_dangerous_str::<Expected>()
        .unwrap_err());
    // expect value
    dbg!(dangerous::input(b"hello")
        .read_all::<_, _, Expected>(|r| r.consume(b"world"))
        .unwrap_err());
}
