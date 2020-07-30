#[test]
fn usage_with_fn() {
    use dangerous::InputError;

    let input = dangerous::input(b"a");
    let mut reader = input.reader();

    fn do_thing<'i>(r: &mut dangerous::Reader<'i>) -> Result<(), InputError> {
        let a = r.read_u8()?;
        assert_eq!(a, b'a');
        r.read_u8()?;
        Ok(())
    }

    assert_eq!(reader.read_all(do_thing), Err(InputError));
}
