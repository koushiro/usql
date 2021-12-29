mod token;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
use core::{iter::Peekable, str::Chars};

pub use self::token::{Literal, Punct, Spacing, TokenStream, TokenTree, Word};
use crate::{
    dialect::{Dialect, DialectLexerConf},
    error::LexerError,
    span::{LineColumn, Span},
};

struct Cursor<'a> {
    rest: &'a str,
    iter: Peekable<Chars<'a>>,
    location: LineColumn,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            rest: input,
            iter: input.chars().peekable(),
            location: LineColumn::default(),
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.iter.next() {
            self.rest = &self.rest[1..];
            self.location.advance(ch);
            Some(ch)
        } else {
            None
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.rest.starts_with(s)
    }

    fn try_next(&mut self, tag: &str) -> bool {
        if self.starts_with(tag) {
            self.rest = &self.rest[tag.len()..];
            self.iter = self.rest.chars().peekable();
            tag.chars().for_each(|ch| self.location.advance(ch));
            true
        } else {
            false
        }
    }

    fn next_if_is(&mut self, ch: char) -> bool {
        if self.iter.next_if_eq(&ch).is_some() {
            self.rest = &self.rest[1..];
            self.location.advance(ch);
            true
        } else {
            false
        }
    }

    fn next_while<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        let mut value = String::new();
        while let Some(ch) = self.iter.next_if(&predicate) {
            value.push(ch);
            self.rest = &self.rest[1..];
            self.location.advance(ch);
        }
        value
    }
}

/// SQL Lexer
pub struct Lexer<'a, D: Dialect> {
    dialect: &'a D,
    cursor: Cursor<'a>,
}

impl<'a, D: Dialect> Lexer<'a, D> {
    /// Creates a new SQL lexer for the given input string.
    pub fn new(dialect: &'a D, input: &'a str) -> Self {
        Self {
            dialect,
            cursor: Cursor::new(input),
        }
    }

    /// Tokenizes the statement and produce a sequence of tokens.
    pub fn tokenize(mut self) -> Result<TokenStream, LexerError> {
        let mut tokens = TokenStream::new();
        loop {
            let start = self.cursor.location;

            let mut token = match self.cursor.peek() {
                Some(&ch) => match ch {
                    ' ' | '\t' | '\n' | '\r' => {
                        self.skip_whitespace();
                        continue;
                    }
                    '-' if self.cursor.try_next("--") => {
                        self.skip_single_line_comment();
                        continue;
                    }
                    '/' if self.cursor.try_next("/*") => {
                        self.skip_multi_line_comment()?;
                        continue;
                    }
                    // national string literal
                    // The spec only allows an uppercase 'N' to introduce a national string literal,
                    // but PostgreSQL/MySQL, at least, allow a lowercase 'n' too.
                    n @ 'N' | n @ 'n' => {
                        self.cursor.next(); // consume the character and check the next one
                        if self.cursor.next_if_is('\'') {
                            // N'...' - <national character string literal>
                            // open quote has been consumed
                            let s = self.tokenize_string_literal('\'')?;
                            TokenTree::Literal(Literal::national_string(s))
                        } else {
                            // regular identifier starting with an "N" or "n"
                            let ident = self.tokenize_ident(n);
                            TokenTree::Word(Word::new::<D::Keyword, _>(ident, None))
                        }
                    }

                    // hex string literal
                    // The spec only allows an uppercase 'X' to introduce a binary string literal,
                    // but PostgreSQL/MySQL, at least, allow a lowercase 'x' too.
                    x @ 'X' | x @ 'x' => {
                        self.cursor.next();
                        if self.cursor.next_if_is('\'') {
                            // X'...' - <hexadecimal character string literal>
                            // open quote has been consumed
                            let s = self.tokenize_string_literal('\'')?;
                            TokenTree::Literal(Literal::hex_string(s))
                        } else {
                            // regular identifier starting with an "X" or "x"
                            let ident = self.tokenize_ident(x);
                            TokenTree::Word(Word::new::<D::Keyword, _>(ident, None))
                        }
                    }
                    // bit string literal
                    // The spec don't allows a 'B' or 'b' to introduce a binary string literal,
                    // but PostgreSQL/MySQL, at least, allow a uppercase 'B' and lowercase 'b'.
                    b @ 'B' | b @ 'b' => {
                        self.cursor.next();
                        if self.cursor.next_if_is('\'') {
                            // B'...' - <binary character string literal>
                            // open quote has been consumed
                            let s = self.tokenize_string_literal('\'')?;
                            TokenTree::Literal(Literal::bit_string(s))
                        } else {
                            // regular identifier starting with an "B" or "b"
                            let ident = self.tokenize_ident(b);
                            TokenTree::Word(Word::new::<D::Keyword, _>(ident, None))
                        }
                    }
                    // string literal
                    quote if self.dialect.lexer_conf().is_string_literal_quotation(quote) => {
                        self.cursor.next(); // consume the open quotation mark of string literal
                        let s = self.tokenize_string_literal(quote)?;
                        TokenTree::Literal(Literal::string(s))
                    }
                    // delimited (quoted) identifier
                    quote
                        if self
                            .dialect
                            .lexer_conf()
                            .is_delimited_identifier_start(quote) =>
                    {
                        self.cursor.next(); // consume the open quotation mark of delimited identifier
                        let ident = self.tokenize_delimited_ident(quote)?;
                        TokenTree::Word(Word::new::<D::Keyword, _>(ident, Some(quote)))
                    }
                    // identifier or keyword
                    ch if self.dialect.lexer_conf().is_identifier_start(ch) => {
                        self.cursor.next(); // consume the identifier start character
                        let ident = self.tokenize_ident(ch);
                        TokenTree::Word(Word::new::<D::Keyword, _>(ident, None))
                    }
                    // number or punct ('.')
                    ch if ch.is_ascii_digit() || ch == '.' => self.tokenize_number(),
                    _ => TokenTree::Punct(self.tokenize_punct()?),
                },
                None => break,
            };

            let end = self.cursor.location;
            token.set_span(Span::with(start, end));
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        self.cursor.next();
    }

    fn skip_single_line_comment(&mut self) {
        let _comment = self.cursor.next_while(|c| c != &'\n');
        if let Some(ch) = self.cursor.next() {
            assert_eq!(ch, '\n');
        }
    }

    fn skip_multi_line_comment(&mut self) -> Result<(), LexerError> {
        let mut nested = 1u32;
        loop {
            match self.cursor.next() {
                Some(ch) => {
                    if ch == '*' && self.cursor.next_if_is('/') {
                        if nested == 1 {
                            break Ok(());
                        } else {
                            nested -= 1;
                        }
                    } else if ch == '/' && self.cursor.next_if_is('*') {
                        nested += 1;
                    }
                }
                None => {
                    return self.tokenize_error("Unexpected EOF while in a multi-line comment");
                }
            }
        }
    }

    fn tokenize_string_literal(&mut self, quote: char) -> Result<String, LexerError> {
        let s = self.cursor.next_while(|&ch| ch != quote);
        // consume the close quote.
        if self.cursor.next() == Some(quote) {
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
        let s = self.cursor.next_while(|&ch| ch != close_quote);
        // consume the close quote.
        if self.cursor.next_if_is(close_quote) {
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
        let rest = self.cursor.next_while(predicate);
        ident.push_str(&rest);
        ident
    }

    fn tokenize_number(&mut self) -> TokenTree {
        // We don't support 0x-prefix syntax, which is a MySQL/MariaDB extension for hex hybrids
        // and behaves as a string or as a number depending on context.
        let mut s = self.cursor.next_while(|ch| ch.is_ascii_digit());

        // match one period
        if self.cursor.next_if_is('.') {
            s.push('.');
        }
        s += &self.cursor.next_while(|ch| ch.is_ascii_digit());

        if s == "." {
            // No number -> Punct ('.')
            let spacing = match self.tokenize_punct_char() {
                Ok(_) => Spacing::Joint,
                Err(_) => Spacing::Alone,
            };
            TokenTree::Punct(Punct::new('.', spacing))
        } else {
            TokenTree::Literal(Literal::number(s))
        }
    }

    fn tokenize_punct(&mut self) -> Result<Punct, LexerError> {
        let ch = self.tokenize_punct_char()?;
        self.cursor.next();
        let spacing = match self.tokenize_punct_char() {
            Ok(_) => Spacing::Joint,
            Err(_) => Spacing::Alone,
        };
        Ok(Punct::new(ch, spacing))
    }

    fn tokenize_punct_char(&mut self) -> Result<char, LexerError> {
        const RECOGNIZED: &str = "~!@#$%^&*()-=+[]{}|;:,<.>/?'";
        match self.cursor.peek() {
            Some(&ch) if RECOGNIZED.contains(ch) => Ok(ch),
            _ => self.tokenize_error("Unexpected EOF or punctuation character"),
        }
    }

    fn tokenize_error<R>(&self, message: impl Into<String>) -> Result<R, LexerError> {
        Err(LexerError {
            message: message.into(),
            location: self.cursor.location,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokenize {
        ($input:expr, $expected:expr) => {{
            let dialect = $crate::ansi::AnsiDialect::default();
            let got = $crate::lexer::Lexer::new(&dialect, $input).tokenize();
            // println!("------------------------------");
            // println!("got = {:?}", got);
            // println!("expected = {:?}", $expected as Result<Vec<Token>, LexerError>);
            // println!("------------------------------");
            assert_eq!(got, $expected);
        }};
        ($input:expr, $expected:expr, $dialect:expr) => {{
            let got = $crate::lexer::Lexer::new($dialect, $input).tokenize();
            // println!("------------------------------");
            // println!("got = {:?}", got);
            // println!("expected = {:?}", $expected  as Result<Vec<Token>, LexerError>);
            // println!("------------------------------");
            assert_eq!(got, $expected);
        }};
    }

    #[test]
    fn tokenize_whitespace() {
        use crate::ansi::AnsiKeyword;
        tokenize!(
            " line1\nline2\t\rline3\r\nline4\r",
            Ok(vec![
                TokenTree::Word(Word::new::<AnsiKeyword, _>("line1", None)),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("line2", None)),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("line3", None)),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("line4", None)),
            ])
        );
    }

    #[test]
    fn tokenize_single_line_comment() {
        // single-line comment
        tokenize!(
            "0--this is single line comment\n1",
            Ok(vec![
                TokenTree::Literal(Literal::number("0")),
                TokenTree::Literal(Literal::number("1")),
            ])
        );

        // single-line comment at eof
        tokenize!(
            "0-- this is single line comment",
            Ok(vec![TokenTree::Literal(Literal::number("0"))])
        );
    }

    #[test]
    fn tokenize_multi_line_comment() {
        tokenize!("/**/", Ok(vec![]));
        tokenize!("/***/", Ok(vec![]));
        tokenize!(
            "/*/*/",
            Err(LexerError {
                message: "Unexpected EOF while in a multi-line comment".into(),
                location: LineColumn::new(1, 5)
            })
        );
        tokenize!("/*line1*/", Ok(vec![]));
        tokenize!("/*line1\nline2*/", Ok(vec![]));
        tokenize!("/*\n--line1\nline2*/", Ok(vec![]));
        tokenize!(
            "/*--line1\nline2",
            Err(LexerError {
                message: "Unexpected EOF while in a multi-line comment".into(),
                location: LineColumn::new(2, 5)
            })
        );
        tokenize!("/*line1\n/*line2*/*/", Ok(vec![]));
        tokenize!("/*line1\n/*line2*/**/", Ok(vec![]));
    }

    #[test]
    fn tokenize_simple_select() {
        use crate::ansi::AnsiKeyword;

        tokenize!(
            "SELECT * FROM customer WHERE id = 1",
            Ok(vec![
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("SELECT").unwrap()),
                TokenTree::Punct(Punct::new('*', Spacing::Alone)),
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("FROM").unwrap()),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("customer", None)),
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("WHERE").unwrap()),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("id", None)),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Literal(Literal::number("1"))
            ])
        );

        tokenize!(
            "SELECT * FROM customer WHERE salary != 'Not Provided'",
            Ok(vec![
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("SELECT").unwrap()),
                TokenTree::Punct(Punct::new('*', Spacing::Alone)),
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("FROM").unwrap()),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("customer", None)),
                TokenTree::Word(Word::keyword::<AnsiKeyword, _>("WHERE").unwrap()),
                TokenTree::Word(Word::new::<AnsiKeyword, _>("salary", None)),
                TokenTree::Punct(Punct::new('!', Spacing::Joint)),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Literal(Literal::string("Not Provided"))
            ])
        );

        // invalid string columns
        tokenize!(
            "\n\nSELECT * FROM table1\tمصطفىh",
            Err(LexerError {
                message: "Unexpected EOF or punctuation character".into(),
                location: LineColumn::new(3, 21)
            })
        )
    }
}
