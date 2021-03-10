use crate::error::{CoreOperation, ExpectedLength};
use crate::input::PrivateExt;

use super::StringReader;

impl<'i, E> StringReader<'i, E> {
    /// Read a char.
    ///
    /// # Errors
    ///
    /// Returns an error if there is no more input.
    #[inline]
    pub fn read_char(&mut self) -> Result<char, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.try_advance(|input| input.split_token(CoreOperation::ReadChar))
    }

    /// Read an optional char.
    ///
    /// Returns `Some(char)` if there was enough input, `None` if not.
    #[inline]
    pub fn read_char_opt(&mut self) -> Option<char> {
        self.advance_opt(PrivateExt::split_token_opt)
    }

    /// Peek the next char in the input without mutating the `Reader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Reader` has no more input.
    #[inline]
    pub fn peek_char(&self) -> Result<char, E>
    where
        E: From<ExpectedLength<'i>>,
    {
        self.input
            .clone()
            .split_token(CoreOperation::PeekChar)
            .map(|(token, _)| token)
    }

    /// Peek the next char in the input without mutating the `Reader`.
    ///
    /// This is equivalent to `peek_char` but does not return an error. Don't use
    /// this function if you want an error if there isn't enough input.
    #[inline]
    #[must_use = "peek result must be used"]
    pub fn peek_char_opt(&self) -> Option<char> {
        self.input.clone().split_token_opt().map(|(token, _)| token)
    }
}
