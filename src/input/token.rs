use super::ByteLength;

/// Implemented for the smallest unit that can be consumed from an [`Input`].
///
/// [`Input`]: crate::Input  
pub trait Token: ByteLength + Copy + 'static {
    /// Returns the token type used in debugging.
    const TYPE: TokenType;
}

/// The token type.
#[derive(Copy, Clone)]
pub enum TokenType {
    /// A byte.
    Byte,
    /// A UTF-8 char.
    Char,
}

impl Token for u8 {
    const TYPE: TokenType = TokenType::Byte;
}

impl Token for char {
    const TYPE: TokenType = TokenType::Char;
}
