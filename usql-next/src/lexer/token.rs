#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::{
    span::Span,
    token::{Keyword, KeywordDef},
};

/// A SQL token stream.
pub type TokenStream = Vec<TokenTree>;

/// A SQL token.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenTree {
    /// A keyword (like SELECT) or an optionally quoted SQL identifier.
    /// Non-reserved keywords are permitted as identifiers without quoting.
    /// Reserved words are permitted as identifiers if you quote them.
    Word(Word),
    /// A single punctuation character (`+`, `,`, `$`, etc.).
    Punct(Punct),

    /// A character string literal (`'hello'`), national character string literal (`N'你好'`),
    /// hexadecimal string literal (X'deadbeef'), bit string literal (B'101010'),
    /// or number literal (`2.3`), etc.
    Literal(Literal),
}

impl fmt::Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenTree::Word(word) => fmt::Display::fmt(word, f),
            TokenTree::Punct(punct) => fmt::Display::fmt(punct, f),
            TokenTree::Literal(literal) => fmt::Display::fmt(literal, f),
        }
    }
}

impl From<Word> for TokenTree {
    fn from(word: Word) -> Self {
        TokenTree::Word(word)
    }
}

impl From<Punct> for TokenTree {
    fn from(punct: Punct) -> Self {
        TokenTree::Punct(punct)
    }
}

impl From<Literal> for TokenTree {
    fn from(literal: Literal) -> Self {
        TokenTree::Literal(literal)
    }
}

impl TokenTree {
    /// Returns the span of this tree, delegating to the `span` method of
    /// the contained token.
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Word(t) => t.span(),
            TokenTree::Punct(t) => t.span(),
            TokenTree::Literal(t) => t.span(),
        }
    }

    /// Configures the span for *only this token*.
    pub fn set_span(&mut self, span: Span) {
        match self {
            TokenTree::Word(t) => t.set_span(span),
            TokenTree::Punct(t) => t.set_span(span),
            TokenTree::Literal(t) => t.set_span(span),
        }
    }
}

/// A keyword (like SELECT) or an optionally quoted SQL identifier
#[derive(Clone)]
pub struct Word {
    /// If the word was not quoted and it matched one of the known keywords,
    /// this will have one of the reserved keyword, otherwise empty.
    keyword: Option<Keyword>,
    /// The value of the token, without the enclosing quotes, and with the
    /// escape sequences (if any) processed.
    value: String,
    /// An identifier can be "quoted" (<delimited identifier> in ANSI parlance).
    /// The standard and most implementations allow using double quotes for this,
    /// but some implementations support other quoting styles as well.
    /// e.g. MySQL also use backtick (`) as the identifier quote character.
    quote: Option<char>,
    span: Span,
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.keyword == other.keyword && self.value == other.value && self.quote == other.quote
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Word")
            .field("keyword", &self.keyword)
            .field("value", &self.value)
            .field("quote", &self.quote)
            .finish()
    }
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

impl Word {
    /// Creates a SQL keyword or an optionally quoted SQL identifier.
    // https://github.com/rust-lang/rust/issues/83701
    pub fn new<K: KeywordDef, W: Into<String>>(value: W, quote: Option<char>) -> Self {
        let value = value.into();
        Self {
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
            span: Span::new(),
        }
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
        keyword.map(|kw| Self {
            keyword: Some(kw),
            value,
            quote: None,
            span: Span::new(),
        })
    }

    /// Returns the span for this word.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Configure the span for this word.
    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}

/// A `Punct` is a single punctuation character like `+`, `-` or `#`.
///
/// Multi-character operators like `+=` are represented as two instances of
/// `Punct` with different forms of `Spacing` returned.
#[derive(Copy, Clone)]
pub struct Punct {
    ch: char,
    spacing: Spacing,
    span: Span,
}

/// Whether a `Punct` is followed immediately by another `Punct` or followed by
/// another token or whitespace.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Spacing {
    /// E.g. `+` is `Alone` in `+ =`, `+ident` or `+()`.
    Alone,
    /// E.g. `+` is `Joint` in `+=` or `'` is `Joint` in `'#`.
    Joint,
}

impl PartialEq for Punct {
    fn eq(&self, other: &Self) -> bool {
        self.ch == other.ch && self.spacing == other.spacing
    }
}

impl fmt::Debug for Punct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Punct")
            .field("ch", &self.ch)
            .field("spacing", &self.spacing)
            .finish()
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.ch, f)
    }
}

impl Punct {
    /// Creates a new `Punct` from the given character and spacing.
    ///
    /// The `ch` argument must be a valid punctuation character permitted by the
    /// language, otherwise the function will panic.
    ///
    /// The returned `Punct` will have the default span of `Span::call_site()`
    /// which can be further configured with the `set_span` method below.
    pub fn new(ch: char, spacing: Spacing) -> Self {
        Self {
            ch,
            spacing,
            span: Span::new(),
        }
    }

    /// Returns the value of this punctuation character as `char`.
    pub fn as_char(&self) -> char {
        self.ch
    }

    /// Returns the spacing of this punctuation character, indicating whether
    /// it's immediately followed by another `Punct` in the token stream, so
    /// they can potentially be combined into a multi-character operator
    /// (`Joint`), or it's followed by some other token or whitespace (`Alone`)
    /// so the operator has certainly ended.
    pub fn spacing(&self) -> Spacing {
        self.spacing
    }

    /// Returns the span for this punctuation character.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Configure the span for this punctuation character.
    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}

/// A character string literal (`'hello'`), national character string literal (`N'你好'`),
/// hexadecimal string literal (X'deadbeef'), bit string literal (B'101010'),
/// or number literal (`2.3`), etc.
#[derive(Clone)]
pub struct Literal {
    inner: LiteralInner,
    span: Span,
}

#[derive(Clone, Debug, PartialEq)]
enum LiteralInner {
    /// Unsigned number literal
    Number(String),
    /// Character string literal
    String(String),
    /// National character string literal
    NationalString(String),
    /// Hexadecimal character string literal
    HexString(String),
    /// Bit string literal
    BitString(String),
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            LiteralInner::Number(ref s) => f.debug_tuple("Number").field(s).finish(),
            LiteralInner::String(ref s) => f.debug_tuple("String").field(s).finish(),
            LiteralInner::NationalString(ref s) => {
                f.debug_tuple("NationalString").field(s).finish()
            }
            LiteralInner::HexString(ref s) => f.debug_tuple("HexString").field(s).finish(),
            LiteralInner::BitString(ref s) => f.debug_tuple("BitString").field(s).finish(),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inner {
            LiteralInner::Number(n) => fmt::Display::fmt(n, f),
            LiteralInner::String(text) => write!(f, "'{}'", text),
            LiteralInner::NationalString(text) => write!(f, "N'{}'", text),
            LiteralInner::HexString(text) => write!(f, "X'{}'", text),
            LiteralInner::BitString(text) => write!(f, "B'{}'", text),
        }
    }
}

impl Literal {
    fn new(inner: LiteralInner) -> Self {
        Self {
            inner,
            span: Span::new(),
        }
    }

    /// Number literal.
    pub fn number(n: impl Into<String>) -> Self {
        Self::new(LiteralInner::Number(n.into()))
    }

    /// String literal.
    pub fn string(string: impl Into<String>) -> Self {
        Self::new(LiteralInner::String(string.into()))
    }

    /// National string literal.
    pub fn national_string(string: impl Into<String>) -> Self {
        Self::new(LiteralInner::NationalString(string.into()))
    }

    /// Hexadecimal string literal.
    pub fn hex_string(string: impl Into<String>) -> Self {
        Self::new(LiteralInner::HexString(string.into()))
    }

    /// Bit string literal.
    pub fn bit_string(string: impl Into<String>) -> Self {
        Self::new(LiteralInner::BitString(string.into()))
    }

    /// Returns the span for this literal.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Configure the span for this literal.
    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }
}
