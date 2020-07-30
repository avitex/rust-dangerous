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
            E: From<EndOfInput<'i>>,
        {
            read_arr!(self, core::mem::size_of::<$ty>()).map(<$ty>::from_le_bytes)
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
            E: From<EndOfInput<'i>>,
        {
            read_arr!(self, core::mem::size_of::<$ty>()).map(<$ty>::from_be_bytes)
        }
    };
}

macro_rules! read_arr {
    ($reader:expr, $size:expr) => {{
        use core::convert::TryInto;
        $reader
            .take($size)
            .map(|i| match i.as_dangerous().try_into() {
                Ok(v) => v,
                Err(_) => unreachable!(),
            })
            .map(Clone::clone)
    }};
}
