#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};
use core::{iter::Peekable, str::Chars};

use usql_core::Dialect;

use crate::{
    error::{LexerError, Location},
    tokens::{Comment, Token, Whitespace},
};

/// SQL Lexer
pub struct Lexer<'a, D: Dialect> {
    iter: Peekable<Chars<'a>>,
    dialect: D,
    location: Location,
}

impl<'a, D: Dialect> Lexer<'a, D> {
    /// Creates a new SQL lexer for the given input string.
    pub fn new(input: &'a str, dialect: D) -> Self {
        Self {
            iter: input.chars().peekable(),
            dialect,
            location: Location::default(),
        }
    }

    ///
    pub fn location(&self) -> Location {
        self.location
    }

    /// Tokenizes the statement and produce a sequence of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token<D::Keyword>>, LexerError> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token()? {
            self.record_location(&token);
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn record_location(&mut self, token: &Token<D::Keyword>) {
        match token {
            Token::Whitespace(Whitespace::Newline) => {
                self.location.line += 1;
                self.location.column = 1;
            }
            Token::Whitespace(Whitespace::Tab) => self.location.column += 4,
            Token::Comment(Comment::SingleLine { .. }) => {
                self.location.line += 1;
                self.location.column = 1;
            }
            Token::Comment(Comment::MultiLine(..)) => {
                todo!()
            }
            Token::Number(s) => self.location.column += s.len() as u64,
            Token::String(s) => self.location.column += s.len() as u64,
            Token::NationalString(s) | Token::BitString(s) | Token::HexString(s) => {
                self.location.column += 1 + s.len() as u64
            }
            Token::Ident(ident) => {
                if ident.quote.is_some() {
                    self.location.column += ident.value.len() as u64 + 2;
                } else {
                    self.location.column += ident.value.len() as u64;
                }
            }
            Token::Keyword(_, keyword) => self.location.column += keyword.len() as u64,
            Token::DoubleColon
            | Token::NotEqual
            | Token::LessThanOrEqual
            | Token::GreaterThanOrEqual
            | Token::LeftShift
            | Token::RightShift
            | Token::DoubleExclamation
            | Token::Concat => self.location.column += 2,
            _ => self.location.column += 1,
        }
    }

    fn next_token(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        match self.iter.peek() {
            Some(&ch) => match ch {
                // whitespace
                ' ' | '\n' | '\t' | '\r' => Ok(self.tokenize_whitespace().map(Token::Whitespace)),
                // national string literal
                'N' => {
                    self.iter.next();
                    match self.iter.peek() {
                        // N'...' - a <national character string literal>
                        Some('\'') => Ok(Some(Token::NationalString(
                            self.tokenize_single_quoted_string()?,
                        ))),
                        // regular identifier starting with an "N"
                        _ => todo!(),
                    }
                }
                // bit string literal
                'B' => {
                    self.iter.next();
                    match self.iter.peek() {
                        // B'...' - a <binary character string literal>
                        Some('\'') => Ok(Some(Token::BitString(
                            self.tokenize_single_quoted_string()?,
                        ))),
                        // regular identifier starting with an "B"
                        _ => todo!(),
                    }
                }
                // hex string literal
                'X' => {
                    self.iter.next();
                    match self.iter.peek() {
                        // X'...' - a <hexadecimal character string literal>
                        Some('\'') => Ok(Some(Token::HexString(
                            self.tokenize_single_quoted_string()?,
                        ))),
                        // regular identifier starting with an "X"
                        _ => todo!(),
                    }
                }
                _ => self.tokenize_symbol(),
            },
            None => Ok(None),
        }
    }

    fn tokenize_whitespace(&mut self) -> Option<Whitespace> {
        self.iter.next().map(|ch| match ch {
            ' ' => Whitespace::Space,
            '\n' => Whitespace::Newline,
            '\t' => Whitespace::Tab,
            '\r' => {
                // Emit a single Whitespace::Newline token for \r and \r\n
                self.next_if(|c| c == '\n');
                Whitespace::Newline
            }
            _ => unreachable!(),
        })
    }

    fn tokenize_single_quoted_string(&mut self) -> Result<String, LexerError> {
        assert!(self.next_if_is('\''));

        let mut s = String::new();
        Ok(s)
    }

    fn tokenize_number(&mut self) -> Result<String, LexerError> {
        todo!()
    }

    fn tokenize_symbol(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        Ok(self
            .next_if_token(|ch| {
                Some(match ch {
                    ',' => Token::Comma,
                    ';' => Token::SemiColon,
                    '.' => Token::Period,
                    ':' => Token::Colon,

                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    '[' => Token::LeftBracket,
                    ']' => Token::RightBracket,
                    '{' => Token::LeftBrace,
                    '}' => Token::RightBrace,

                    '=' => Token::Equal,
                    '<' => Token::LessThan,
                    '>' => Token::GreaterThan,

                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Asterisk,
                    '/' => Token::Slash,
                    '%' => Token::Percent,

                    '^' => Token::Caret,
                    '!' => Token::Exclamation,
                    '?' => Token::Question,
                    '~' => Token::Tilde,
                    '&' => Token::Ampersand,
                    '|' => Token::Pipe,
                    '\\' => Token::Backslash,
                    '#' => Token::Sharp,
                    '@' => Token::At,
                    _ => Token::Other(ch),
                })
            })
            .map(|token| match token {
                Token::Colon if self.next_if_is(':') => Token::DoubleColon,
                Token::LessThan if self.next_if_is('>') => Token::NotEqual,
                Token::LessThan if self.next_if_is('=') => Token::LessThanOrEqual,
                Token::LessThan if self.next_if_is('<') => Token::LeftShift,
                Token::GreaterThan if self.next_if_is('=') => Token::GreaterThanOrEqual,
                Token::GreaterThan if self.next_if_is('>') => Token::RightShift,
                Token::Minus if self.next_if_is('-') => {
                    Token::Comment(self.tokenize_single_line_comment("--"))
                }
                Token::Exclamation if self.next_if_is('=') => Token::NotEqual,
                Token::Exclamation if self.next_if_is('!') => Token::DoubleExclamation,
                Token::Pipe if self.next_if_is('|') => Token::Concat,
                token => token,
            }))
    }

    /// Tokenizes single-line comment and returns the comment.
    fn tokenize_single_line_comment(&mut self, prefix: impl Into<String>) -> Comment {
        let comment = self.next_while(|c| c != '\n').unwrap_or_default();
        Comment::SingleLine {
            prefix: prefix.into(),
            comment,
        }
    }

    /// Grabs the next character if it matches the predicate function
    fn next_if<F: Fn(char) -> bool>(&mut self, predicate: F) -> Option<char> {
        self.iter.peek().filter(|&c| predicate(*c))?;
        self.iter.next()
    }

    /// Consumes the next character if it matches the character `ch`, and returns true if it matches.
    #[inline]
    fn next_if_is(&mut self, ch: char) -> bool {
        self.next_if(|c| c == ch).is_some()
    }

    /// Grabs the next single-character token if the tokenizer function returns one
    fn next_if_token<F: Fn(char) -> Option<Token<D::Keyword>>>(
        &mut self,
        tokenizer: F,
    ) -> Option<Token<D::Keyword>> {
        let token = self.iter.peek().and_then(|&c| tokenizer(c))?;
        self.iter.next();
        Some(token)
    }

    /// Grabs the next characters that match the predicate, as a string
    fn next_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> Option<String> {
        let mut value = String::new();
        while let Some(c) = self.next_if(&predicate) {
            value.push(c)
        }
        Some(value).filter(|v| !v.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
