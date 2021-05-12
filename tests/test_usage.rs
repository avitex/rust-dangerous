#[test]
fn test_usage_with_fn() {
    use dangerous::error::{Expected, RetryRequirement, ToRetryRequirement};
    use dangerous::{BytesReader, Input};

    let input = dangerous::input(b"a");

    fn do_thing<'i>(r: &mut BytesReader<'i, Expected<'i>>) -> Result<(), Expected<'i>> {
        let a = r.read_u8()?;
        assert_eq!(a, b'a');
        r.read_u8()?;
        Ok(())
    }

    assert_eq!(
        input.read_all(do_thing).unwrap_err().to_retry_requirement(),
        RetryRequirement::new(1)
    );
}

#[test]
fn test_streaming_usage_with_fatal_requirement() {
    use dangerous::{BytesReader, Input, Invalid};

    let input = dangerous::input(b"blah");
    let result: Result<_, Invalid> = input.read_all(|r| {
        r.take(2)?
            .read_all(|r: &mut BytesReader<Invalid>| r.skip(4))?;
        r.consume(b"ah")
    });

    assert_eq!(result, Err(Invalid::fatal()));
}

#[test]
fn test_streaming_usage_with_valid_requirement() {
    use dangerous::error::{Invalid, RetryRequirement, ToRetryRequirement};
    use dangerous::Input;

    let input = dangerous::input(b"blah");
    let result: Result<_, Invalid> = input.read_all(|r| {
        r.skip(2)?;
        r.take(4)
    });

    assert_eq!(
        result.unwrap_err().to_retry_requirement(),
        RetryRequirement::new(2)
    );
}

#[test]
fn test_retry_unbound_spent_reader() {
    use dangerous::error::{RetryRequirement, ToRetryRequirement};
    use dangerous::{BytesReader, Error, Input, Invalid};

    fn parse<'i, E: Error<'i>>(r: &mut BytesReader<'i, E>) -> Result<(), E> {
        // This may have not finished so the input is unbound.
        let _ = r.take_while(|_| true);
        // This may have not finished so the input is unbound.
        let unbound = r.take_while(|_| true);
        // We try to take more and get a retry error.
        unbound.read_all(|r| r.skip(1))
    }

    let input = dangerous::input(b"blah");
    let result: Result<_, Invalid> = input.read_all(parse);

    assert_eq!(
        result.unwrap_err().to_retry_requirement(),
        RetryRequirement::new(1),
    );
}

#[test]
fn test_retry_footgun_with_take_consumed() {
    use dangerous::{BytesReader, Error, Input, Invalid};

    fn parse<'i, E: Error<'i>>(r: &mut BytesReader<'i, E>) -> Result<(), E> {
        let (_, consumed) = r.try_take_consumed(|r| {
            // We take a exact length of input
            r.consume(b"blah")
        })?;
        consumed.read_all(|r| r.consume(b"blah1"))
    }

    let input = dangerous::input(b"blah");
    let result: Result<_, Invalid> = input.read_all(parse);

    assert_eq!(result, Err(Invalid::fatal()));
}

#[test]
fn test_retry_footgun_from_unbound_spent_reader_take_0() {
    use dangerous::{BytesReader, Error, Input, Invalid};

    fn parse<'i, E: Error<'i>>(r: &mut BytesReader<'i, E>) -> Result<(), E> {
        let _ = r.take_while(|_| true);
        // We read input with a length of zero from the spent Reader.
        let bound = r.take(0)?;
        // This should result in a fatal error.
        bound.read_all(|r| r.skip(1))
    }

    let input = dangerous::input(b"blah");
    let result: Result<_, Invalid> = input.read_all(parse);

    assert_eq!(result, Err(Invalid::fatal()));
}
