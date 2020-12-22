macro_rules! impl_read_num {
    ($ty:ident, le: $read_le:ident, be: $read_be:ident) => {
        impl_read_num!($ty, stringify!($ty), le: $read_le, be: $read_be);
    };
    ($ty:ident, $ty_str:expr, le: $read_le:ident, be: $read_be:ident) => {
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
            read_num!(self, E, $ty, concat!("little-endian ", $ty_str), from_le_bytes)
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
            read_num!(self, E, $ty, concat!("big-endian ", $ty_str), from_be_bytes)
        }
    };
}

macro_rules! read_num {
    ($reader:expr, $err_ty:ident, $num_ty:ident, $expected:expr, $from_xx_bytes:ident) => {{
        $reader.try_advance(|input| {
            let (arr, next) = split_arr!(input, $num_ty, concat!("read ", $expected))?;
            let number = <$num_ty>::$from_xx_bytes(arr);
            Ok((number, next))
        })
    }};
}

macro_rules! split_arr {
    ($input:expr, u8, $expected:expr) => {
        $input.split_arr_1($expected)
    };
    ($input:expr, u16, $expected:expr) => {
        $input.split_arr_2($expected)
    };
    ($input:expr, u32, $expected:expr) => {
        $input.split_arr_4($expected)
    };
    ($input:expr, u64, $expected:expr) => {
        $input.split_arr_8($expected)
    };
    ($input:expr, u128, $expected:expr) => {
        $input.split_arr_16($expected)
    };
    ($input:expr, i8, $expected:expr) => {
        $input.split_arr_1($expected)
    };
    ($input:expr, i16, $expected:expr) => {
        $input.split_arr_2($expected)
    };
    ($input:expr, i32, $expected:expr) => {
        $input.split_arr_4($expected)
    };
    ($input:expr, i64, $expected:expr) => {
        $input.split_arr_8($expected)
    };
    ($input:expr, i128, $expected:expr) => {
        $input.split_arr_16($expected)
    };
    ($input:expr, f32, $expected:expr) => {
        $input.split_arr_4($expected)
    };
    ($input:expr, f64, $expected:expr) => {
        $input.split_arr_8($expected)
    };
}

macro_rules! forward_fmt {
    (impl Debug for $name:ident) => {
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                crate::display::fmt::DebugBase::fmt(self, f)
            }
        }
    };
    (impl Display for $name:ident) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                crate::display::fmt::DisplayBase::fmt(self, f)
            }
        }
    };
    (impl<$($impl_generics:tt),+> Debug for $name:ident<$($ty_generics:tt),+> $(where $bname:tt:$bvalue:tt$(<$($b_generics:tt),+>)?)?) => {
        impl<$($impl_generics),+> core::fmt::Debug for $name<$($ty_generics),+> $(where $bname:$bvalue$(<$($b_generics),+>)?)? {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                crate::display::fmt::DebugBase::fmt(self, f)
            }
        }
    };
    (impl<$($impl_generics:tt),+> Display for $name:ident<$($ty_generics:tt),+> $(where $bname:tt:$bvalue:tt$(<$($b_generics:tt),+>)?)?) => {
        impl<$($impl_generics),+> core::fmt::Display for $name<$($ty_generics),+> $(where $bname:$bvalue$(<$($b_generics),+>)?)? {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                crate::display::fmt::DisplayBase::fmt(self, f)
            }
        }
    };
}
