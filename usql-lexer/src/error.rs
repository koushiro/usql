#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

/// Lexer error
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub line: u64,
    pub col: u64,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at Line: {}, Column {}",
            self.message, self.line, self.col
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LexerError {}
