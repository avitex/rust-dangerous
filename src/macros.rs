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
            E: From<ExpectedLength<'i>>,
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
            E: From<ExpectedLength<'i>>,
        {
            read_num!(self, E, $ty, concat!("big-endian ", $ty_str), from_be_bytes)
        }
    };
}

macro_rules! read_num {
    ($reader:expr, $err_ty:ident, $num_ty:ty, $num_desc:expr, $from_xx_bytes:ident) => {{
        $reader.context_mut(concat!("read ", $num_desc), |r| {
            r.input
                .split_at::<$err_ty>(core::mem::size_of::<$num_ty>())
                .map(
                    |(head, tail)| match core::convert::TryInto::try_into(head.as_dangerous()) {
                        Ok(arr) => {
                            r.input = tail;
                            <$num_ty>::$from_xx_bytes(arr)
                        }
                        Err(_) => unreachable!(),
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

        impl<'i> Error<'i> for $name<'i> {
            fn with_context<C>(mut self, input: &'i Input, _context: C) -> Self
            where
                C: Context,
            {
                self.update_input(input);
                self
            }
        }

        impl<'i> From<$name<'i>> for Invalid {
            fn from(err: $name<'i>) -> Self {
                Invalid {
                    retry_requirement: err.retry_requirement(),
                }
            }
        }

        #[cfg(feature = "std")]
        impl<'i> std::error::Error for $name<'i> {}
    };
}
