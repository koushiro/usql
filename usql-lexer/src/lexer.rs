#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{iter::Peekable, str::Chars};

use usql_core::{Dialect, DialectLexerConf};

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

    /// Returns the current location scanned by lexer.
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
                // The spec only allows an uppercase 'N' to introduce a national string literal,
                // but PostgreSQL/MySQL, at least, allow a lowercase 'n' too.
                n @ 'N' | n @ 'n' => {
                    if self.next_if_is('\'') {
                        // N'...' - <national character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::NationalString(s)))
                    } else {
                        // regular identifier starting with an "N" or "n"
                        let ident = self.tokenize_ident(n);
                        Ok(Some(Token::ident(ident, None)))
                    }
                }
                // hex string literal
                // The spec only allows an uppercase 'X' to introduce a binary string literal,
                // but PostgreSQL/MySQL, at least, allow a lowercase 'x' too.
                x @ 'X' | x @ 'x' => {
                    if self.next_if_is('\'') {
                        // X'...' - <hexadecimal character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::HexString(s)))
                    } else {
                        // regular identifier starting with an "X" or "x"
                        let ident = self.tokenize_ident(x);
                        Ok(Some(Token::ident(ident, None)))
                    }
                }
                // bit string literal
                // The spec don't allows an 'B' or 'b' to introduce a binary string literal,
                // but PostgreSQL/MySQL, at least, allow a uppercase 'B' and lowercase 'b'.
                b @ 'B' | b @ 'b' => {
                    if self.next_if_is('\'') {
                        // B'...' - <binary character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::BitString(s)))
                    } else {
                        // regular identifier starting with an "B" or "b"
                        let ident = self.tokenize_ident(b);
                        Ok(Some(Token::ident(ident, None)))
                    }
                }
                // string literal
                quote if self.dialect.lexer_conf().is_string_literal_quotation(quote) => {
                    self.iter.next(); // consume the open quotation mark of string literal
                    let s = self.tokenize_string_literal(quote)?;
                    Ok(Some(Token::String(s)))
                }
                // delimited (quoted) identifier
                quote
                    if self
                        .dialect
                        .lexer_conf()
                        .is_delimited_identifier_start(quote) =>
                {
                    self.iter.next(); // consume the open quotation mark of delimited identifier
                    let ident = self.tokenize_delimited_ident(quote)?;
                    Ok(Some(Token::ident(ident, Some(quote))))
                }
                // identifier or keyword
                ch if self.dialect.lexer_conf().is_identifier_start(ch) => {
                    self.iter.next();
                    let ident = self.tokenize_ident(ch);
                    Ok(Some(Token::ident(ident, None)))
                }
                // number or period
                ch if ch.is_ascii_digit() || ch == '.' => self.tokenize_number(),
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
                self.iter.next_if(|c| c == &'\n');
                Whitespace::Newline
            }
            _ => unreachable!(),
        })
    }

    fn tokenize_string_literal(&mut self, quote: char) -> Result<String, LexerError> {
        let s = self.next_while(|&ch| ch != quote);
        // consume the close quote.
        if self.iter.next() == Some(quote) {
            Ok(s)
        } else {
            self.tokenize_error("Unterminated string literal")
        }
    }

    fn tokenize_delimited_ident(&mut self, open_quote: char) -> Result<String, LexerError> {
        let close_quote = match open_quote {
            '"' => '"', // ANSI and most dialects
            '`' => '`', // MySQL
            _ => return self.tokenize_error("Unexpected quoting style"),
        };
        let s = self.next_while(|&ch| ch != close_quote);
        // consume the close quote.
        if self.iter.next() == Some(close_quote) {
            Ok(s)
        } else {
            self.tokenize_error(format!(
                "Expected close delimiter '{}' before EOF",
                close_quote
            ))
        }
    }

    fn tokenize_ident(&mut self, first: char) -> String {
        let mut ident = first.to_string();
        let predicate = |ch: &char| self.dialect.lexer_conf().is_identifier_part(*ch);
        ident.push_str(&next_while(&mut self.iter, predicate));
        ident
    }

    fn tokenize_number(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        todo!()
    }

    fn tokenize_symbol(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        Ok(self
            .next_if_token(|ch| {
                Some(match ch {
                    ',' => Token::Comma,
                    ';' => Token::SemiColon,
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
        let comment = self.next_while(|c| c != &'\n');
        Comment::SingleLine {
            prefix: prefix.into(),
            comment,
        }
    }

    fn tokenize_error<R>(&self, message: impl Into<String>) -> Result<R, LexerError> {
        Err(self.location.into_error(message))
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

    /// Consumes the next character if it matches the character `ch`, and returns true if it matches.
    #[inline]
    fn next_if_is(&mut self, ch: char) -> bool {
        self.iter.next_if_eq(&ch).is_some()
    }

    /// Grabs the next characters that match the predicate, as a string
    fn next_while<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        next_while(&mut self.iter, predicate)
    }
}

fn next_while<F: Fn(&char) -> bool>(chars: &mut Peekable<Chars<'_>>, predicate: F) -> String {
    let mut value = String::new();
    while let Some(c) = chars.next_if(&predicate) {
        value.push(c)
    }
    value
}

#[cfg(test)]
mod tests {}
