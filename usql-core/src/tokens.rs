#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

use crate::dialect::KeywordDef;

/// SQL token
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Token<K> {
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
    /// Bit string literal: i.e.: B'101010'.
    BitString(String),
    /// Hexadecimal string literal: i.e.: X'deadbeef'.
    HexString(String),

    /// An optionally quoted SQL identifier.
    Ident(Ident),
    /// A keyword.
    Keyword(K, &'static str),

    /// A character that could not be tokenized.
    Other(char),

    /// Comma `,`
    Comma,
    /// SemiColon `;`
    SemiColon,
    /// Period `.`
    Period,
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
    LessThan,
    /// Less than or equal `<=`
    LessThanOrEqual,
    /// Greater than `>`
    GreaterThan,
    /// Greater than or equal `>=`
    GreaterThanOrEqual,

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
}

impl<K: fmt::Display> fmt::Display for Token<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Whitespace(space) => write!(f, "{}", space),
            Token::Comment(comment) => write!(f, "{}", comment),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "{}", s),
            Token::NationalString(s) => write!(f, "M'{}'", s),
            Token::BitString(s) => write!(f, "B'{}'", s),
            Token::HexString(s) => write!(f, "X'{}'", s),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Keyword(keyword, _) => write!(f, "{}", keyword),
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
            Token::LessThan => f.write_str("<"),
            Token::LessThanOrEqual => f.write_str("<="),
            Token::GreaterThan => f.write_str(">"),
            Token::GreaterThanOrEqual => f.write_str(">="),
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
            Token::Other(c) => write!(f, "{}", c),
        }
    }
}

impl<K: KeywordDef> Token<K> {
    /// Creates a SQL keyword.
    pub fn keyword(keyword: impl AsRef<str>) -> Option<Self> {
        let keyword_uppercase = keyword.as_ref().to_uppercase();
        K::KEYWORD_STRINGS
            .binary_search(&keyword_uppercase.as_str())
            .map(|x| Self::Keyword(K::KEYWORDS[x].clone(), K::KEYWORD_STRINGS[x]))
            .ok()
    }

    /// Creates an optionally quoted SQL identifier.
    pub fn ident(value: impl Into<String>, quote: Option<char>) -> Self {
        Self::Ident(Ident {
            value: value.into(),
            quote,
        })
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
        comment: String
    },
    // TODO: Not support now
    /// Multiple line comment.
    MultiLine(String),
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SingleLine { prefix, comment } => write!(f, "{} {}", prefix, comment),
            Self::MultiLine(s) => write!(f, "/*{}*/", s),
        }
    }
}

/// An optionally quoted SQL identifier
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ident {
    /// The value of the token, without the enclosing quotes, and with the
    /// escape sequences (if any) processed.
    pub value: String,
    /// An identifier can be "quoted" (<delimited identifier> in ANSI parlance).
    /// The standard and most implementations allow using double quotes for this,
    /// but some implementations support other quoting styles as well.
    /// e.g. MySQL also use backtick (`) as the identifier quote character.
    pub quote: Option<char>,
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.quote {
            None => f.write_str(&self.value),
            Some(q) if q == '"' || q == '`' => write!(f, "{}{}{}", q, self.value, q),
            Some(q) => panic!("Unsupported quote character {} for SQL identifier!", q),
        }
    }
}
