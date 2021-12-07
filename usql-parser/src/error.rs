#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use core::fmt;

use usql_lexer::LexerError;

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

/// A help function to create a parser error.
pub(crate) fn parse_error<R>(message: impl Into<String>) -> Result<R, ParserError> {
    Err(ParserError::ParseError(message.into()))
}

/// A help function to create a parse error that indicates unexpected EOF.
#[allow(unused)]
pub(crate) fn unexpected_eof<R>() -> Result<R, ParserError> {
    Err(ParserError::ParseError("Unexpected EOF".into()))
}
