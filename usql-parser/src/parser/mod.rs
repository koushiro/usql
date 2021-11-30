mod expression;
mod statement;
mod types;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::Display;

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
    iter: MultiPeek<Box<dyn Iterator<Item = Token> + 'static>>,
}

impl<'a, D: Dialect> Parser<'a, D> {
    /// Creates a new SQL parser with the given tokens.
    pub fn new_with_tokens(dialect: &'a D, tokens: Vec<Token>) -> Self {
        // ignore whitespace and comment.
        let filter = tokens
            .into_iter()
            .filter(|token| !token.is_whitespace() && !token.is_comment());
        Self {
            dialect,
            iter: (Box::new(filter) as Box<dyn Iterator<Item = Token>>).multipeek(),
        }
    }

    /// Creates a new SQL parser with the given sql string.
    pub fn new_with_sql(dialect: &'a D, sql: &str) -> Result<Self, ParserError> {
        let tokens = Lexer::new(dialect, sql).tokenize()?;
        Ok(Self::new_with_tokens(dialect, tokens))
    }

    /// Parse a comma-separated list of 1+ items accepted by `F`.
    pub fn parse_comma_separated<T, F>(&mut self, mut f: F) -> Result<Vec<T>, ParserError>
    where
        F: FnMut(&mut Parser<'a, D>) -> Result<T, ParserError>,
    {
        let mut values = vec![];
        loop {
            values.push(f(self)?);
            if !self.next_token_if_is(&Token::Comma) {
                break;
            }
        }
        Ok(values)
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
            parse_error(format!("Expected: {}, but not found", expected))
        }
    }

    /// Consumes the next keyword token and return ok if it matches the expected
    /// keyword, otherwise return error.
    pub fn expect_keyword(&mut self, expected: Keyword) -> Result<(), ParserError> {
        if self.parse_keyword(expected) {
            Ok(())
        } else {
            let found = self.peek_token().cloned();
            self.expected(expected, found)
        }
    }

    /// Consumes the next keyword tokens if they matches the expected keywords, otherwise return error.
    pub fn expect_keywords(&mut self, expected: &[Keyword]) -> Result<(), ParserError> {
        for &kw in expected {
            self.expect_keyword(kw)?;
        }
        Ok(())
    }

    /// Consumes the next keyword token if the token is one of the expected keywords.
    pub fn expect_one_of_keywords(&mut self, keywords: &[Keyword]) -> Result<Keyword, ParserError> {
        if let Some(keyword) = self.parse_one_of_keywords(keywords) {
            Ok(keyword)
        } else {
            let found = self.peek_token().cloned();
            self.expected(format!("one of {:?}", keywords), found)
        }
    }

    /// Consumes the next keyword token and return true if it matches the expected
    /// keyword, otherwise return false.
    pub fn parse_keyword(&mut self, keyword: Keyword) -> bool {
        self.next_token_if(|token| token.is_keyword(keyword))
            .is_some()
    }

    /// Consumes the next multiple keyword tokens and return true if they matches the
    /// expected keywords, otherwise return false.
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
        for _ in 0..keywords.len() {
            self.next_token();
        }
        true
    }

    /// Consumes the next keyword token if the token is one of the expected keywords.
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

    /// Consumes the next token and return ok if it matches the expected token,
    /// otherwise return error.
    pub fn expect_token(&mut self, expected: &Token) -> Result<(), ParserError> {
        if self.next_token_if_is(expected) {
            Ok(())
        } else {
            let found = self.peek_token().cloned();
            self.expected(expected, found)
        }
    }

    /// Returns a reference to the next_token() value without advancing the iterator.
    ///
    /// Like [`next_token`], if there is a value, it is wrapped in a `Some(Token)`.
    /// But if the iteration is over, `None` is returned.
    pub fn peek_token(&mut self) -> Option<&Token> {
        self.iter.peek()
    }

    /// Works exactly like `.next_token()` with the only difference that it
    /// doesn't advance itself.
    /// `.peek_next_token()` can be called multiple times, to peek further ahead.
    /// When `.next_token()` is called, reset the peeking "cursor".
    pub fn peek_next_token(&mut self) -> Option<&Token> {
        self.iter.peek_next()
    }

    /// Reset the peek cursor.
    pub fn reset_peek_cursor(&mut self) {
        self.iter.reset_cursor();
    }

    /// Consumes the next token and return the token.
    pub fn next_token(&mut self) -> Option<Token> {
        self.iter.next()
    }

    /// Consumes the next token and return the token if it `func` return true,
    /// otherwise return None.
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
        self.iter.next_if_eq(expected).is_some()
    }
}
