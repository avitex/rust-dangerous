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

/// Read a byte.
///
/// # Errors
///
/// Returns an error if there is no more input.
#[inline]
pub fn read_u8(&mut self) -> Result<u8, E>
where
    E: From<ExpectedLength<'i>>,
{
    self.try_advance(|input| input.split_first("read u8"))
}

/// Read a `i8`.
///
/// # Errors
///
/// Returns an error if there is no more input.
#[inline]
pub fn read_i8(&mut self) -> Result<u8, E>
where
    E: From<ExpectedLength<'i>>,
{
    self.try_advance(|input| input.split_first("read i8") as i8)
}

impl_read_num!(u16, le: read_u16_le, be: read_u16_be);
impl_read_num!(i16, le: read_i16_le, be: read_i16_be);
impl_read_num!(u32, le: read_u32_le, be: read_u32_be);
impl_read_num!(i32, le: read_i32_le, be: read_i32_be);
impl_read_num!(u64, le: read_u64_le, be: read_u64_be);
impl_read_num!(i64, le: read_i64_le, be: read_i64_be);
impl_read_num!(u128, le: read_u128_le, be: read_u128_be);
impl_read_num!(i128, le: read_i128_le, be: read_i128_be);
impl_read_num!(f32, le: read_f32_le, be: read_f32_be);
impl_read_num!(f64, le: read_f64_le, be: read_f64_be);
