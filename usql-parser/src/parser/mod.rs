mod ddl;
mod dml;
mod dql;
mod tcl;

/*
use core::iter::Peekable;

use usql_lexer::{Lexer, Token};

/// SQL Parser
pub struct Parser<'a, K> {
    tokens: Peekable<Token<K>>,
}

impl<'a, K> Parser<'a, K> {
    /// Creates a new SQL parser for the given input string
    pub fn new(input: &'a str, keyword: &'a K) -> Parser<'a, K> {
        Self {
            tokens: Lexer::new(input, keyword).peekable(),
        }
    }
}
*/
