#[test]
fn usage_with_fn() {
    use dangerous::Invalid;

    let input = dangerous::input(b"a");

    fn do_thing<'i>(r: &mut dangerous::Reader<'i, Invalid>) -> Result<(), Invalid> {
        let a = r.read_u8()?;
        assert_eq!(a, b'a');
        r.read_u8()?;
        Ok(())
    }

    assert_eq!(input.read_all(do_thing), Err(Invalid::new(1)));
}
