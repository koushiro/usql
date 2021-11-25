mod expression;
mod statement;
mod types;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::String,
    vec::{IntoIter, Vec},
};
use core::fmt::Display;
#[cfg(feature = "std")]
use std::vec::IntoIter;

use usql_core::{Dialect, Keyword};
use usql_lexer::{Lexer, Token};

use crate::{
    error::{parse_error, ParserError},
    peek::{MultiPeek, PeekIteratorExt},
};

/// SQL Parser
pub struct Parser<'a, D: Dialect> {
    #[allow(unused)]
    dialect: &'a D,
    iter: MultiPeek<IntoIter<Token>>,
}

impl<'a, D: Dialect> Parser<'a, D> {
    /// Creates a new SQL parser with the given tokens.
    pub fn new_with_tokens(dialect: &'a D, tokens: Vec<Token>) -> Self {
        Self {
            dialect,
            iter: tokens.into_iter().multipeek(),
        }
    }

    /// Creates a new SQL parser with the given sql string.
    pub fn new_with_sql(dialect: &'a D, sql: &str) -> Result<Self, ParserError> {
        let tokens = Lexer::new(dialect, sql).tokenize()?;
        Ok(Self::new_with_tokens(dialect, tokens))
    }

    /// Report unexpected token.
    pub fn expected<R>(
        &self,
        expected: impl Display,
        found: Option<impl Display>,
    ) -> Result<R, ParserError> {
        if let Some(found) = found {
            parse_error(format!("Expected: {}, found: {}", expected, found))
        } else {
            parse_error(format!("Expected: {}, found: EOF", expected))
        }
    }

    #[doc(hidden)]
    pub fn expect_keyword(&mut self, expected: Keyword) -> Result<(), ParserError> {
        if self.parse_keyword(expected) {
            Ok(())
        } else {
            let found = self.peek_token().cloned();
            self.expected(expected, found)
        }
    }

    #[doc(hidden)]
    pub fn expect_keywords(&mut self, expected: &[Keyword]) -> Result<(), ParserError> {
        for &kw in expected {
            self.expect_keyword(kw)?;
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn parse_keyword(&mut self, keyword: Keyword) -> bool {
        self.next_token_if(|token| token.is_keyword(keyword))
            .is_some()
    }

    #[doc(hidden)]
    pub fn parse_keywords(&mut self, keywords: &[Keyword]) -> bool {
        for &keyword in keywords {
            if let Some(token) = self.peek_next_token() {
                if !token.is_keyword(keyword) {
                    // reset cursor and return immediately
                    self.reset_peek_cursor();
                    return false;
                }
            } else {
                // reset cursor and return immediately
                self.reset_peek_cursor();
                return false;
            }
        }
        true
    }

    #[doc(hidden)]
    pub fn parse_one_of_keywords(&mut self, keywords: &[Keyword]) -> Option<Keyword> {
        match self.peek_token() {
            Some(token) => {
                if let Some(keyword) = token.is_one_of_keywords(keywords) {
                    self.next_token();
                    Some(keyword)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn expect_token(&mut self, expected: &Token) -> Result<(), ParserError> {
        if self.next_token_if_is(expected) {
            Ok(())
        } else {
            let found = self.peek_token().cloned();
            self.expected(expected, found)
        }
    }

    #[doc(hidden)]
    pub fn peek_token(&mut self) -> Option<&Token> {
        self.iter.peek()
    }

    #[doc(hidden)]
    pub fn peek_next_token(&mut self) -> Option<&Token> {
        self.iter.peek_next()
    }

    #[doc(hidden)]
    pub fn reset_peek_cursor(&mut self) {
        self.iter.reset_cursor();
    }

    #[doc(hidden)]
    pub fn next_token(&mut self) -> Option<Token> {
        self.iter.next()
    }

    #[doc(hidden)]
    pub fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> Option<Token> {
        self.iter.next_if(func)
    }

    /// Consumes the next token and return the token if it matches the expected
    /// token, otherwise return None.
    pub fn next_token_if_eq(&mut self, expected: &Token) -> Option<Token> {
        self.iter.next_if_eq(expected)
    }

    /// Consumes the next token and return true if it matches the expected token,
    /// otherwise return false.
    pub fn next_token_if_is(&mut self, expected: &Token) -> bool {
        self.iter.next_if_is(expected)
    }
}
