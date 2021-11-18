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
            if self.dialect.lexer_conf().ignore_whitespace() {
                if let Token::Whitespace(_) = token {
                    continue;
                }
            }
            if self.dialect.lexer_conf().ignore_comment() {
                if let Token::Comment(_) = token {
                    continue;
                }
            }
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
                    self.iter.next(); // consume the character and check the next one
                    if self.next_if_is('\'') {
                        // N'...' - <national character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::NationalString(s)))
                    } else {
                        // regular identifier starting with an "N" or "n"
                        let ident = self.tokenize_ident(n);
                        Ok(Some(Token::make(ident, None)))
                    }
                }
                // hex string literal
                // The spec only allows an uppercase 'X' to introduce a binary string literal,
                // but PostgreSQL/MySQL, at least, allow a lowercase 'x' too.
                x @ 'X' | x @ 'x' => {
                    self.iter.next(); // consume the character and check the next one
                    if self.next_if_is('\'') {
                        // X'...' - <hexadecimal character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::HexString(s)))
                    } else {
                        // regular identifier starting with an "X" or "x"
                        let ident = self.tokenize_ident(x);
                        Ok(Some(Token::make(ident, None)))
                    }
                }
                // bit string literal
                // The spec don't allows an 'B' or 'b' to introduce a binary string literal,
                // but PostgreSQL/MySQL, at least, allow a uppercase 'B' and lowercase 'b'.
                b @ 'B' | b @ 'b' => {
                    self.iter.next(); // consume the character and check the next one
                    if self.next_if_is('\'') {
                        // B'...' - <binary character string literal>
                        // open quote has been consumed
                        let s = self.tokenize_string_literal('\'')?;
                        Ok(Some(Token::BitString(s)))
                    } else {
                        // regular identifier starting with an "B" or "b"
                        let ident = self.tokenize_ident(b);
                        Ok(Some(Token::make(ident, None)))
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
                    Ok(Some(Token::make(ident, Some(quote))))
                }
                // identifier or keyword
                ch if self.dialect.lexer_conf().is_identifier_start(ch) => {
                    self.iter.next(); // consume the identifier start character
                    let ident = self.tokenize_ident(ch);
                    Ok(Some(Token::make(ident, None)))
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
        let rest = next_while(&mut self.iter, predicate);
        ident.push_str(&rest);
        ident
    }

    fn tokenize_number(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        let mut s = self.next_while(|ch| ch.is_ascii_digit());

        // We don't support 0xvalue syntax, which is a MySQL/MariaDB extension for hex hybrids
        // and behaves as a string or as a number depending on context.

        // match one period
        if self.next_if_is('.') {
            s.push('.');
        }
        s += &self.next_while(|ch| ch.is_ascii_digit());

        // No number -> Token::Period
        if s == "." {
            return Ok(Some(Token::Period));
        }
        Ok(Some(Token::Number(s)))
    }

    fn tokenize_symbol(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        let token = self.next_if_token(|ch| {
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
                _ => Token::Char(ch),
            })
        });
        if let Some(token) = token {
            Ok(Some(match token {
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
        } else {
            Ok(None)
        }
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
        value.push(c);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokenize {
        ($input:expr, $expected:expr) => {{
            let dialect = ::usql_core::ansi::AnsiDialect::default();
            let mut lexer = $crate::Lexer::new($input, dialect);
            let got = lexer.tokenize();
            // println!("------------------------------");
            // println!("got = {:?}", $got);
            // println!("expected = {:?}", $expected);
            // println!("------------------------------");
            assert_eq!(got, $expected);
        }};
        ($input:expr, $expected:expr, $dialect:expr) => {{
            let mut lexer = $crate::Lexer::new($input, $dialect);
            let got = lexer.tokenize();
            // println!("------------------------------");
            // println!("got = {:?}", $got);
            // println!("expected = {:?}", $expected);
            // println!("------------------------------");
            assert_eq!(got, $expected);
        }};
    }

    #[test]
    fn tokenize_whitespace() {
        tokenize!(
            " line1\nline2\t\rline3\r\nline4\r",
            Ok(vec![
                Token::Whitespace(Whitespace::Space),
                Token::ident("line1", None),
                Token::Whitespace(Whitespace::Newline),
                Token::ident("line2", None),
                Token::Whitespace(Whitespace::Tab),
                Token::Whitespace(Whitespace::Newline),
                Token::ident("line3", None),
                Token::Whitespace(Whitespace::Newline),
                Token::ident("line4", None),
                Token::Whitespace(Whitespace::Newline),
            ])
        );
    }

    #[test]
    fn tokenize_comment() {
        // single-line comment
        tokenize!(
            "0--this is single line comment\n1",
            Ok(vec![
                Token::Number("0".into()),
                Token::Comment(Comment::SingleLine {
                    prefix: "--".into(),
                    comment: "this is single line comment".into()
                }),
                Token::Whitespace(Whitespace::Newline),
                Token::Number("1".into())
            ])
        );

        // single-line comment at eof
        tokenize!(
            "0-- this is single line comment",
            Ok(vec![
                Token::Number("0".into()),
                Token::Comment(Comment::SingleLine {
                    prefix: "--".into(),
                    comment: " this is single line comment".into()
                }),
            ])
        );

        // TODO: multi-line comment
    }

    #[test]
    fn tokenize_number_literal() {
        tokenize!(
            "1234567890 0987654321",
            Ok(vec![
                Token::Number("1234567890".into()),
                Token::Whitespace(Whitespace::Space),
                Token::Number("0987654321".into()),
            ])
        );

        tokenize!(
            ".1 12345.6789 0. .",
            Ok(vec![
                Token::Number(".1".into()),
                Token::Whitespace(Whitespace::Space),
                Token::Number("12345.6789".into()),
                Token::Whitespace(Whitespace::Space),
                Token::Number("0.".into()),
                Token::Whitespace(Whitespace::Space),
                Token::Period,
            ])
        );
    }

    #[test]
    fn tokenize_string_literal() {
        tokenize!("'hello'", Ok(vec![Token::String("hello".into())]));
        tokenize!("'hello'", Ok(vec![Token::String("hello".into())]));

        tokenize!("N'你好'", Ok(vec![Token::NationalString("你好".into())]));
        tokenize!("n'你好'", Ok(vec![Token::NationalString("你好".into())]));

        tokenize!("X'abcdef'", Ok(vec![Token::HexString("abcdef".into())]));
        tokenize!("x'abcdef'", Ok(vec![Token::HexString("abcdef".into())]));

        tokenize!("B'01010101'", Ok(vec![Token::BitString("01010101".into())]));
        tokenize!("b'01010101'", Ok(vec![Token::BitString("01010101".into())]));

        // newline in string literal
        tokenize!(
            "'foo\r\nbar\nbaz'",
            Ok(vec![Token::String("foo\r\nbar\nbaz".into())])
        );

        // invalid string literal
        tokenize!(
            "\nمصطفىh",
            Ok(vec![
                Token::Whitespace(Whitespace::Newline),
                Token::Char('م'),
                Token::Char('ص'),
                Token::Char('ط'),
                Token::Char('ف'),
                Token::Char('ى'),
                Token::ident("h", None),
            ])
        );

        // unterminated string literal
        tokenize!(
            "select 'foo",
            Err(Location { line: 1, column: 8 }.into_error("Unterminated string literal"))
        );
    }

    #[test]
    fn tokenize_delimited_ident() {
        tokenize!("\"foo\"", Ok(vec![Token::ident("foo", Some('\"'))]));

        // mismatch quotes
        tokenize!(
            "\"foo",
            Err(Location { line: 1, column: 1 }
                .into_error("Expected close delimiter '\"' before EOF"))
        );
    }

    #[test]
    fn tokenize_string_concat() {
        tokenize!(
            "SELECT 'a' || 'b'",
            Ok(vec![
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::String("a".into()),
                Token::Whitespace(Whitespace::Space),
                Token::Concat,
                Token::Whitespace(Whitespace::Space),
                Token::String("b".into()),
            ])
        );
    }

    #[test]
    fn tokenize_bitwise_op() {
        tokenize!(
            "SELECT one | two ^ three",
            Ok(vec![
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("one").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::Pipe,
                Token::Whitespace(Whitespace::Space),
                Token::ident("two", None),
                Token::Whitespace(Whitespace::Space),
                Token::Caret,
                Token::Whitespace(Whitespace::Space),
                Token::ident("three", None),
            ])
        )
    }

    #[test]
    fn tokenize_mysql_logical_xor() {
        tokenize!(
            "SELECT true XOR true, false XOR false, true XOR false, false XOR true",
            Ok(vec![
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("true").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("XOR").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("true").unwrap(),
                Token::Comma,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("false").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("XOR").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("false").unwrap(),
                Token::Comma,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("true").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("XOR").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("false").unwrap(),
                Token::Comma,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("false").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("XOR").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("true").unwrap(),
            ]),
            usql_core::mysql::MysqlDialect::default()
        );
    }

    #[test]
    fn tokenize_simple_select() {
        tokenize!(
            "SELECT * FROM customer WHERE id = 1",
            Ok(vec![
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::Asterisk,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("FROM").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::ident("customer", None),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("WHERE").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::ident("id", None),
                Token::Whitespace(Whitespace::Space),
                Token::Equal,
                Token::Whitespace(Whitespace::Space),
                Token::Number("1".into()),
            ])
        );

        tokenize!(
            "SELECT * FROM customer WHERE salary != 'Not Provided'",
            Ok(vec![
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::Asterisk,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("FROM").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::ident("customer", None),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("WHERE").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::ident("salary", None),
                Token::Whitespace(Whitespace::Space),
                Token::NotEqual,
                Token::Whitespace(Whitespace::Space),
                Token::String("Not Provided".into()),
            ])
        );

        // invalid string columns
        tokenize!(
            "\n\nSELECT * FROM table\tمصطفىh",
            Ok(vec![
                Token::Whitespace(Whitespace::Newline),
                Token::Whitespace(Whitespace::Newline),
                Token::keyword("SELECT").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::Asterisk,
                Token::Whitespace(Whitespace::Space),
                Token::keyword("FROM").unwrap(),
                Token::Whitespace(Whitespace::Space),
                Token::keyword("table").unwrap(),
                Token::Whitespace(Whitespace::Tab),
                Token::Char('م'),
                Token::Char('ص'),
                Token::Char('ط'),
                Token::Char('ف'),
                Token::Char('ى'),
                Token::ident("h", None),
            ])
        )
    }
}
