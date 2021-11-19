#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

/// Location info for input.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[doc(hidden)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line: {}, Column: {}", self.line, self.column)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl Location {
    pub(crate) fn into_error(self, message: impl Into<String>) -> LexerError {
        LexerError {
            message: message.into(),
            location: self,
        }
    }
}

/// Lexer error
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LexerError {
    /// The specified error message.
    pub message: String,
    /// The location info of error message.
    pub location: Location,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LexerError {}
