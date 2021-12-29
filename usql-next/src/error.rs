#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use core::fmt;

use crate::span::LineColumn;

/// Lexer error
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LexerError {
    /// The specified error message.
    pub message: String,
    /// The location info of error message.
    pub location: LineColumn,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LexerError {}

/// Parser error
#[derive(Clone, Debug, PartialEq)]
pub enum ParserError {
    /// Tokenize error.
    TokenizeError(String),
    /// Parse error.
    ParseError(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParserError::TokenizeError(s) => s,
            ParserError::ParseError(s) => s,
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParserError {}

impl From<LexerError> for ParserError {
    fn from(err: LexerError) -> Self {
        Self::TokenizeError(err.to_string())
    }
}

impl From<String> for ParserError {
    fn from(err: String) -> Self {
        Self::ParseError(err)
    }
}

impl From<&str> for ParserError {
    fn from(err: &str) -> Self {
        Self::ParseError(err.into())
    }
}
