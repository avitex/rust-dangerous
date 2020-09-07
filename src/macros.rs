macro_rules! impl_read_num {
    ($ty:ty, le: $read_le:ident, be: $read_be:ident) => {
        impl_read_num!($ty, stringify!($ty), le: $read_le, be: $read_be);
    };
    ($ty:ty, $ty_str:expr, le: $read_le:ident, be: $read_be:ident) => {
        #[doc = "Read a little-endian encoded `"]
        #[doc = $ty_str]
        #[doc = "` from the reader."]
        ///
        /// # Errors
        ///
        /// Returns an error if there is not sufficient input left to read.
        pub fn $read_le(&mut self) -> Result<$ty, E>
        where
            E: FromError<E>,
            E: FromError<ExpectedLength<'i>>,
        {
            read_num!(self, E, $ty, concat!("little-endian ", $ty_str), from_le_bytes)
        }

        #[doc = "Read a big-endian encoded `"]
        #[doc = $ty_str]
        #[doc = "` from the reader."]
        ///
        /// # Errors
        ///
        /// Returns an error if there is not sufficient input left to read.
        pub fn $read_be(&mut self) -> Result<$ty, E>
        where
            E: FromError<E>,
            E: FromError<ExpectedLength<'i>>,
        {
            read_num!(self, E, $ty, concat!("big-endian ", $ty_str), from_be_bytes)
        }
    };
}

macro_rules! read_num {
    ($reader:expr, $err_ty:ident, $num_ty:ty, $num_desc:expr, $from_xx_bytes:ident) => {{
        $reader
            .input
            .split_at::<$err_ty>(core::mem::size_of::<$num_ty>())
            .map(
                |(head, tail)| match core::convert::TryInto::try_into(head.as_dangerous()) {
                    Ok(arr) => {
                        $reader.input = tail;
                        <$num_ty>::$from_xx_bytes(arr)
                    }
                    Err(_) => unreachable!(),
                },
            )
            .map_err(|err| {
                E::from_err_ctx(
                    err,
                    SealedContext {
                        operation: concat!("read ", $num_desc),
                        input: $reader.input,
                    },
                )
            })
    }};
}

macro_rules! impl_error {
    ($name:ident) => {
        impl<'i> fmt::Display for $name<'i> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.display().fmt(f)
            }
        }

        #[cfg(feature = "std")]
        impl<'i> std::error::Error for $name<'i> {}
    };
}
