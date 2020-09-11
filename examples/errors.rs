use dangerous::Expected;

fn main() {
    dbg!(dangerous::input(b"A\xC3\xA9 \xC2")
        .to_dangerous_str::<Expected>()
        .unwrap_err());

    dbg!(dangerous::input(b"A\xC3\xA9 \xFF")
        .to_dangerous_str::<Expected>()
        .unwrap_err());
}
