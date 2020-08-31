use dangerous::Expected;

fn main() {
    let input = dangerous::input(b"A\xC3\xA9 \xC2");
    println!("{:#}", input.to_dangerous_str::<Expected>().unwrap_err());

    let input = dangerous::input(b"A\xC3\xA9 \xFF");
    println!("{:#}", input.to_dangerous_str::<Expected>().unwrap_err());
}
