macro_rules! impl_read_num {
    ($ty:ident, le: ($read_le:ident, $op_le:expr), be: ($read_be:ident, $op_be:expr)) => {
        impl_read_num!(
            $ty,
            stringify!($ty),
            le: ($read_le, $op_le),
            be: ($read_be, $op_be)
        );
    };
    ($ty:ident, $ty_str:expr, le: ($read_le:ident, $op_le:expr), be: ($read_be:ident, $op_be:expr)) => {
        #[doc = "Read a little-endian encoded `"]
        #[doc = $ty_str]
        #[doc = "`."]
        ///
        /// # Errors
        ///
        /// Returns an error if there is not sufficient input left to read.
        pub fn $read_le(&mut self) -> Result<$ty, E>
        where
            E: From<ExpectedLength<'i>>,
        {
            read_num!(self, E, $ty, $op_le, from_le_bytes)
        }

        #[doc = "Read a big-endian encoded `"]
        #[doc = $ty_str]
        #[doc = "`."]
        ///
        /// # Errors
        ///
        /// Returns an error if there is not sufficient input left to read.
        pub fn $read_be(&mut self) -> Result<$ty, E>
        where
            E: From<ExpectedLength<'i>>,
        {
            read_num!(self, E, $ty, $op_le, from_be_bytes)
        }
    };
}

macro_rules! read_num {
    ($reader:expr, $err_ty:ident, $num_ty:ident, $op:expr, $from_xx_bytes:ident) => {{
        $reader.try_advance(|input| {
            let (arr, next) = split_array!(input, $num_ty, $op)?;
            let number = <$num_ty>::$from_xx_bytes(arr);
            Ok((number, next))
        })
    }};
}

macro_rules! split_array {
    ($input:expr, u8, $expected:expr) => {
        $input.split_array_1($expected)
    };
    ($input:expr, u16, $expected:expr) => {
        $input.split_array_2($expected)
    };
    ($input:expr, u32, $expected:expr) => {
        $input.split_array_4($expected)
    };
    ($input:expr, u64, $expected:expr) => {
        $input.split_array_8($expected)
    };
    ($input:expr, u128, $expected:expr) => {
        $input.split_array_16($expected)
    };
    ($input:expr, i8, $expected:expr) => {
        $input.split_array_1($expected)
    };
    ($input:expr, i16, $expected:expr) => {
        $input.split_array_2($expected)
    };
    ($input:expr, i32, $expected:expr) => {
        $input.split_array_4($expected)
    };
    ($input:expr, i64, $expected:expr) => {
        $input.split_array_8($expected)
    };
    ($input:expr, i128, $expected:expr) => {
        $input.split_array_16($expected)
    };
    ($input:expr, f32, $expected:expr) => {
        $input.split_array_4($expected)
    };
    ($input:expr, f64, $expected:expr) => {
        $input.split_array_8($expected)
    };
}

macro_rules! for_common_array_sizes {
    ($impl:ident) => {
        for_common_array_sizes!($impl:
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 32, 64, 128, 256,
            512, 1024, 2048, 4096
        );
    };
    ($impl:ident: $($n:expr),*) => {
        $(
            $impl!($n);
        )*
    }
}
