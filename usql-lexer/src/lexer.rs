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
    dialect: &'a D,
    iter: Peekable<Chars<'a>>,
    location: Location,
}

impl<'a, D: Dialect> Lexer<'a, D> {
    /// Creates a new SQL lexer for the given input string.
    pub fn new(dialect: &'a D, input: &'a str) -> Self {
        Self {
            dialect,
            iter: input.chars().peekable(),
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
            if self.dialect.lexer_conf().ignore_whitespace() && token.is_whitespace() {
                continue;
            }
            if self.dialect.lexer_conf().ignore_comment() && token.is_comment() {
                continue;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token<D::Keyword>>, LexerError> {
        match self.iter.peek() {
            Some(&ch) => match ch {
                // whitespace
                ' ' | '\t' | '\n' | '\r' => Ok(self.tokenize_whitespace().map(Token::Whitespace)),
                // national string literal
                // The spec only allows an uppercase 'N' to introduce a national string literal,
                // but PostgreSQL/MySQL, at least, allow a lowercase 'n' too.
                n @ 'N' | n @ 'n' => {
                    self.next_char(); // consume the character and check the next one
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
                    self.next_char(); // consume the character and check the next one
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
                // The spec don't allows a 'B' or 'b' to introduce a binary string literal,
                // but PostgreSQL/MySQL, at least, allow a uppercase 'B' and lowercase 'b'.
                b @ 'B' | b @ 'b' => {
                    self.next_char(); // consume the character and check the next one
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
                    self.next_char(); // consume the open quotation mark of string literal
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
                    self.next_char(); // consume the open quotation mark of delimited identifier
                    let ident = self.tokenize_delimited_ident(quote)?;
                    Ok(Some(Token::make(ident, Some(quote))))
                }
                // identifier or keyword
                ch if self.dialect.lexer_conf().is_identifier_start(ch) => {
                    self.next_char(); // consume the identifier start character
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
            ' ' => {
                self.location.column += 1;
                Whitespace::Space
            }
            '\t' => {
                self.location.column += 1;
                Whitespace::Tab
            }
            '\n' => {
                self.location.line += 1;
                self.location.column = 1;
                Whitespace::Newline
            }
            '\r' => {
                // Emit a single Whitespace::Newline token for \r and \r\n
                self.iter.next_if_eq(&'\n');
                self.location.line += 1;
                self.location.column = 1;
                Whitespace::Newline
            }
            _ => unreachable!(),
        })
    }

    fn tokenize_string_literal(&mut self, quote: char) -> Result<String, LexerError> {
        let s = self.next_while(|&ch| ch != quote);
        // consume the close quote.
        if self.next_char() == Some(quote) {
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
        if self.next_if_is(close_quote) {
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
        let rest = next_while(&mut self.location, &mut self.iter, predicate);
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
                Token::Slash if self.next_if_is('*') => {
                    Token::Comment(self.tokenize_multi_line_comment()?)
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
        let mut comment = self.next_while(|c| c != &'\n');
        if let Some(ch) = self.next_char() {
            assert_eq!(ch, '\n');
            comment.push(ch);
        }
        Comment::SingleLine {
            prefix: prefix.into(),
            comment,
        }
    }

    /// Tokenize multi-line comment and returns the comment.
    fn tokenize_multi_line_comment(&mut self) -> Result<Comment, LexerError> {
        let mut comment = String::new();
        let mut nested = 1;
        loop {
            match self.next_char() {
                Some(ch) => {
                    if ch == '*' && self.next_if_is('/') {
                        if nested == 1 {
                            let lines = comment.split('\n').map(|s| s.to_string()).collect();
                            break Ok(Comment::MultiLine(lines));
                        } else {
                            nested -= 1;
                            comment.push_str("*/");
                        }
                    } else if ch == '/' && self.next_if_is('*') {
                        nested += 1;
                        comment.push_str("/*");
                    } else {
                        comment.push(ch);
                    }
                }
                None => {
                    return self.tokenize_error("Unexpected EOF while in a multi-line comment");
                }
            }
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
        self.next_char();
        Some(token)
    }

    /// Consumes the next character and records the current location.
    fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.iter.next() {
            self.location.advance(ch);
            Some(ch)
        } else {
            None
        }
    }

    /// Consumes the next character and records the current location
    /// if it matches the character `ch`, and returns true if it matches.
    #[inline]
    fn next_if_is(&mut self, ch: char) -> bool {
        if self.iter.next_if_eq(&ch).is_some() {
            self.location.advance(ch);
            true
        } else {
            false
        }
    }

    /// Grabs the next characters that match the predicate, as a string
    fn next_while<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        next_while(&mut self.location, &mut self.iter, predicate)
    }
}

fn next_while<F: Fn(&char) -> bool>(
    loc: &mut Location,
    chars: &mut Peekable<Chars<'_>>,
    predicate: F,
) -> String {
    let mut value = String::new();
    while let Some(ch) = chars.next_if(&predicate) {
        loc.advance(ch);
        value.push(ch);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokenize {
        ($input:expr, $expected:expr) => {{
            let dialect = ::usql_core::ansi::AnsiDialect::default();
            let mut lexer = $crate::Lexer::new(&dialect, $input);
            let got = lexer.tokenize();
            // println!("------------------------------");
            // println!("got = {:?}", $got);
            // println!("expected = {:?}", $expected);
            // println!("------------------------------");
            assert_eq!(got, $expected);
        }};
        ($input:expr, $expected:expr, $dialect:expr) => {{
            let mut lexer = $crate::Lexer::new($dialect, $input);
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
    fn tokenize_single_line_comment() {
        // single-line comment
        tokenize!(
            "0--this is single line comment\n1",
            Ok(vec![
                Token::Number("0".into()),
                Token::Comment(Comment::SingleLine {
                    prefix: "--".into(),
                    comment: "this is single line comment\n".into()
                }),
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
    }

    #[test]
    fn tokenize_multi_line_comment() {
        tokenize!(
            "/**/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec!["".into()]))])
        );
        tokenize!(
            "/***/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec!["*".into()]))])
        );
        tokenize!(
            "/*/*/",
            Err(Location { line: 1, column: 6 }
                .into_error("Unexpected EOF while in a multi-line comment"))
        );
        tokenize!(
            "/*line1*/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec![
                "line1".into()
            ]))])
        );
        tokenize!(
            "/*line1\nline2*/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec![
                "line1".into(),
                "line2".into(),
            ]))])
        );
        tokenize!(
            "/*\n--line1\nline2*/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec![
                "".into(),
                "--line1".into(),
                "line2".into()
            ]))])
        );
        tokenize!(
            "/*--line1\nline2",
            Err(Location { line: 2, column: 6 }
                .into_error("Unexpected EOF while in a multi-line comment"))
        );
        tokenize!(
            "/*line1\n/*line2*/*/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec![
                "line1".into(),
                "/*line2*/".into()
            ]))])
        );
        tokenize!(
            "/*line1\n/*line2*/**/",
            Ok(vec![Token::Comment(Comment::MultiLine(vec![
                "line1".into(),
                "/*line2*/*".into()
            ]))])
        );
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
            Err(Location {
                line: 1,
                column: 12
            }
            .into_error("Unterminated string literal"))
        );
    }

    #[test]
    fn tokenize_delimited_ident() {
        tokenize!("\"foo\"", Ok(vec![Token::ident("foo", Some('\"'))]));

        // mismatch quotes
        tokenize!(
            "\"foo",
            Err(Location { line: 1, column: 5 }
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
        let dialect = usql_core::mysql::MysqlDialect::default();
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
            &dialect
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
