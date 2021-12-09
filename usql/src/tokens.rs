#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::keywords::{Keyword, KeywordDef};

/// SQL token
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Token {
    /// Whitespace (space, newline, tab).
    Whitespace(Whitespace),
    /// Single-line comment or multi-line comment.
    Comment(Comment),

    /// An unsigned numeric literal.
    Number(String),

    /// Character string literal: i.e: 'string'
    String(String),
    /// National character string literal: i.e: N'string'.
    NationalString(String),
    /// Hexadecimal string literal: i.e.: X'deadbeef'.
    HexString(String),
    /// Bit string literal: i.e.: B'101010'. (Not ANSI SQL)
    BitString(String),

    /// A keyword (like SELECT) or an optionally quoted SQL identifier.
    /// Non-reserved keywords are permitted as identifiers without quoting.
    /// Reserved words are permitted as identifiers if you quote them.
    Word(Word),

    /// Period `.`
    Period,
    /// Comma `,`
    Comma,
    /// SemiColon `;`
    SemiColon,
    /// Colon `:`
    Colon,
    /// Double colon `::`
    DoubleColon,

    /// Left parenthesis `(`
    LeftParen,
    /// Right parenthesis `)`
    RightParen,
    /// Left bracket `[`
    LeftBracket,
    /// Right bracket `]`
    RightBracket,
    /// Left brace `{`
    LeftBrace,
    /// Right brace `}`
    RightBrace,

    /// Equal `=`
    Equal,
    /// Not equal `<>` or `!=`
    NotEqual,
    /// Less than `<`
    Less,
    /// Less than or equal `<=`
    LessOrEqual,
    /// Greater than `>`
    Greater,
    /// Greater than or equal `>=`
    GreaterOrEqual,

    /// Left Shift `<<`
    LeftShift,
    /// Right Shift `>>`
    RightShift,

    /// Plus `+`
    Plus,
    /// Minus `-`
    Minus,
    /// Asterisk `*`
    Asterisk,
    /// Slash `/`
    Slash,
    /// Percent `%`
    Percent,

    /// Caret `^`
    Caret,
    /// Exclamation `!`
    Exclamation,
    /// Double exclamation `!!`
    DoubleExclamation,
    /// Question `?`
    Question,
    /// Tilde `~`
    Tilde,
    /// Ampersand `&`
    Ampersand,
    /// Pipe `|`
    Pipe,
    /// Concat `||`
    Concat,
    /// Backslash `\`
    Backslash,
    /// Sharp `#`
    Sharp,
    /// At `@`
    At,

    /// A character that could not be tokenized.
    Char(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Whitespace(space) => write!(f, "{}", space),
            Token::Comment(comment) => write!(f, "{}", comment),
            Token::Number(n) => f.write_str(n),
            Token::String(s) => f.write_str(s),
            Token::NationalString(s) => write!(f, "M'{}'", s),
            Token::BitString(s) => write!(f, "B'{}'", s),
            Token::HexString(s) => write!(f, "X'{}'", s),
            Token::Word(word) => write!(f, "{}", word),
            Token::Comma => f.write_str(","),
            Token::SemiColon => f.write_str(";"),
            Token::Period => f.write_str("."),
            Token::Colon => f.write_str(":"),
            Token::DoubleColon => f.write_str("::"),
            Token::LeftParen => f.write_str("("),
            Token::RightParen => f.write_str(")"),
            Token::LeftBracket => f.write_str("["),
            Token::RightBracket => f.write_str("]"),
            Token::LeftBrace => f.write_str("{"),
            Token::RightBrace => f.write_str("}"),
            Token::Equal => f.write_str("="),
            Token::NotEqual => f.write_str("<>"),
            Token::Less => f.write_str("<"),
            Token::LessOrEqual => f.write_str("<="),
            Token::Greater => f.write_str(">"),
            Token::GreaterOrEqual => f.write_str(">="),
            Token::LeftShift => f.write_str("<<"),
            Token::RightShift => f.write_str(">>"),
            Token::Plus => f.write_str("+"),
            Token::Minus => f.write_str("-"),
            Token::Asterisk => f.write_str("*"),
            Token::Slash => f.write_str("/"),
            Token::Percent => f.write_str("%"),
            Token::Caret => f.write_str("^"),
            Token::Exclamation => f.write_str("!"),
            Token::DoubleExclamation => f.write_str("!!"),
            Token::Question => f.write_str("?"),
            Token::Tilde => f.write_str("~"),
            Token::Ampersand => f.write_str("&"),
            Token::Pipe => f.write_str("|"),
            Token::Concat => f.write_str("||"),
            Token::Backslash => f.write_str("\\"),
            Token::Sharp => f.write_str("#"),
            Token::At => f.write_str("@"),
            Token::Char(c) => write!(f, "{}", c),
        }
    }
}

impl Token {
    /// Creates a SQL keyword or an optionally quoted SQL identifier.
    // https://github.com/rust-lang/rust/issues/83701
    pub fn word<K: KeywordDef, W: Into<String>>(value: W, quote: Option<char>) -> Self {
        let value = value.into();
        Self::Word(Word {
            keyword: if quote.is_none() {
                let keyword_uppercase = value.to_uppercase();
                K::KEYWORDS_STRING
                    .binary_search(&keyword_uppercase.as_str())
                    .map(|x| K::KEYWORDS[x])
                    .ok()
            } else {
                None
            },
            value,
            quote,
        })
    }

    /// Creates a SQL keyword.
    // https://github.com/rust-lang/rust/issues/83701
    pub fn keyword<K: KeywordDef, W: Into<String>>(value: W) -> Option<Self> {
        let value = value.into();
        let keyword_uppercase = value.to_uppercase();
        let keyword = K::KEYWORDS_STRING
            .binary_search(&keyword_uppercase.as_str())
            .map(|x| K::KEYWORDS[x])
            .ok();
        keyword.map(|kw| {
            Self::Word(Word {
                keyword: Some(kw),
                value,
                quote: None,
            })
        })
    }

    /// Checks if the token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace(_))
    }

    /// Checks if the token is comment.
    pub fn is_comment(&self) -> bool {
        matches!(self, Token::Comment(_))
    }

    /// Checks if the token is keyword.
    #[inline]
    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        matches!(self, Token::Word(w) if w.keyword == Some(keyword))
    }

    /// Checks if the token is keyword, which is one of the `keywords`.
    pub fn is_one_of_keywords(&self, keywords: &[Keyword]) -> Option<Keyword> {
        if let Token::Word(w) = self {
            if let Some(keyword) = w.keyword {
                if keywords.contains(&keyword) {
                    return Some(keyword);
                }
            }
        }
        None
    }
}

/// Whitespace token
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Whitespace {
    Space,
    Newline,
    Tab,
}

impl fmt::Display for Whitespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Space => f.write_str(" "),
            Self::Newline => f.write_str("\n"),
            Self::Tab => f.write_str("\t"),
        }
    }
}

/// Comment token
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Comment {
    /// Single line comment.
    SingleLine {
        /// The prefix of the comment.
        prefix: String,
        /// The comment text.
        comment: String,
    },
    /// Multiple line comment.
    MultiLine(Vec<String>),
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SingleLine { prefix, comment } => write!(f, "{}{}", prefix, comment),
            Self::MultiLine(lines) => {
                f.write_str("/*")?;
                let mut delim = "";
                for line in lines {
                    write!(f, "{}", delim)?;
                    delim = "\n";
                    write!(f, "{}", line)?;
                }
                f.write_str("*/")?;
                Ok(())
            }
        }
    }
}

/// A keyword (like SELECT) or an optionally quoted SQL identifier
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Word {
    /// If the word was not quoted and it matched one of the known keywords,
    /// this will have one of the reserved keyword, otherwise empty.
    pub keyword: Option<Keyword>,
    /// The value of the token, without the enclosing quotes, and with the
    /// escape sequences (if any) processed.
    pub value: String,
    /// An identifier can be "quoted" (<delimited identifier> in ANSI parlance).
    /// The standard and most implementations allow using double quotes for this,
    /// but some implementations support other quoting styles as well.
    /// e.g. MySQL also use backtick (`) as the identifier quote character.
    pub quote: Option<char>,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.quote {
            None => f.write_str(&self.value),
            Some(q) if q == '"' || q == '`' => write!(f, "{}{}{}", q, self.value, q),
            Some(q) => panic!("Unsupported quote character {} for SQL identifier!", q),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_display() {
        let comment = Comment::SingleLine {
            prefix: "--".into(),
            comment: "this is single line comment".into(),
        };
        assert_eq!(comment.to_string(), "--this is single line comment");

        let comment = Comment::MultiLine(vec!["line1".into(), "line2".into()]);
        assert_eq!(comment.to_string(), "/*line1\nline2*/");
    }
}
