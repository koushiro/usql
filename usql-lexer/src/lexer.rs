#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};
use core::{iter::Peekable, str::Chars};

use crate::{error::LexerError, tokens::Token};

struct Location {
    line: u64,
    col: u64,
}

impl Location {
    fn new() -> Self {
        Self { line: 1, col: 1 }
    }

    fn into_error(self, message: impl Into<String>) -> LexerError {
        LexerError {
            message: message.into(),
            line: self.line,
            col: self.col,
        }
    }
}

/// SQL Lexer
pub struct Lexer<'a> {
    iter: Peekable<Chars<'a>>,
    loc: Location,
}

impl<'a> Lexer<'a> {
    /// Creates a new SQL lexer for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            iter: input.chars().peekable(),
            loc: Location::new(),
        }
    }

    /// Tokenizes the statement and produce a sequence of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
