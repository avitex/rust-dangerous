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
            self.try_advance(|input| {
                let (arr, next) = input.split_array($op_le)?;
                let number = $ty::from_le_bytes(arr);
                Ok((number, next))
            })
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
            self.try_advance(|input| {
                let (arr, next) = input.split_array($op_be)?;
                let number = $ty::from_be_bytes(arr);
                Ok((number, next))
            })
        }
    };
}
