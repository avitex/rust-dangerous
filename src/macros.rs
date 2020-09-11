macro_rules! split_arr {
    ($input:expr, u8, $op:expr) => {
        $input.split_arr_1($op)
    };
    ($input:expr, u16, $op:expr) => {
        $input.split_arr_2($op)
    };
    ($input:expr, u32, $op:expr) => {
        $input.split_arr_4($op)
    };
    ($input:expr, u64, $op:expr) => {
        $input.split_arr_8($op)
    };
    ($input:expr, u128, $op:expr) => {
        $input.split_arr_16($op)
    };
    ($input:expr, i8, $op:expr) => {
        $input.split_arr_1($op)
    };
    ($input:expr, i16, $op:expr) => {
        $input.split_arr_2($op)
    };
    ($input:expr, i32, $op:expr) => {
        $input.split_arr_4($op)
    };
    ($input:expr, i64, $op:expr) => {
        $input.split_arr_8($op)
    };
    ($input:expr, i128, $op:expr) => {
        $input.split_arr_16($op)
    };
    ($input:expr, f32, $op:expr) => {
        $input.split_arr_4($op)
    };
    ($input:expr, f64, $op:expr) => {
        $input.split_arr_8($op)
    };
}

macro_rules! impl_read_num {
    ($ty:ident, le: $read_le:ident, be: $read_be:ident) => {
        impl_read_num!($ty, stringify!($ty), le: $read_le, be: $read_be);
    };
    ($ty:ident, $ty_str:expr, le: $read_le:ident, be: $read_be:ident) => {
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
            read_num!(self, E, $ty, concat!("read little-endian ", $ty_str), from_le_bytes)
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
            read_num!(self, E, $ty, concat!("read big-endian ", $ty_str), from_be_bytes)
        }
    };
}

macro_rules! read_num {
    ($reader:expr, $err_ty:ident, $num_ty:ident, $operation:expr, $from_xx_bytes:ident) => {{
        let (arr, tail) = split_arr!($reader.input, $num_ty, $operation)?;
        $reader.input = tail;
        Ok(<$num_ty>::$from_xx_bytes(arr))
    }};
}

macro_rules! impl_error_common {
    ($name:ident) => {
        impl<'i> fmt::Display for $name<'i> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.display().fmt(f)
            }
        }

        impl<'i> fmt::Debug for $name<'i> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.display().fmt(f)
            }
        }

        impl<'i> From<$name<'i>> for crate::error::Invalid {
            fn from(err: $name<'i>) -> Self {
                err.to_retry_requirement().into()
            }
        }

        #[cfg(feature = "std")]
        impl<'i> std::error::Error for $name<'i> {}
    };
}
