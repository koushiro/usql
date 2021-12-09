#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
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
    pub(crate) fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.column = 1;
            self.line += 1;
        } else {
            self.column += 1;
        }
    }

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
