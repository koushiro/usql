use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

/// A simple customizable SQL dialect structure.
#[derive(Clone, Debug)]
pub struct CustomDialect<K, L, P> {
    _keyword: PhantomData<K>,
    lexer_conf: L,
    parser_conf: P,
}

impl<K, L: Default, P: Default> Default for CustomDialect<K, L, P> {
    fn default() -> Self {
        Self {
            _keyword: PhantomData,
            lexer_conf: L::default(),
            parser_conf: P::default(),
        }
    }
}

impl<K: KeywordDef, L: DialectLexerConf, P: DialectParserConf> CustomDialect<K, L, P> {
    /// Creates a new SQL Dialect.
    pub fn new(lexer_conf: L, parser_conf: P) -> Self {
        Self {
            _keyword: PhantomData,
            lexer_conf,
            parser_conf,
        }
    }
}

impl<K: KeywordDef, L: DialectLexerConf, P: DialectParserConf> Dialect for CustomDialect<K, L, P> {
    type Keyword = K;
    type LexerConf = L;
    type ParserConf = P;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

/// The marker for a dialect.
pub trait Dialect: Debug {
    /// The keyword definition of the dialect.
    type Keyword: KeywordDef;

    /// The lexer configuration of the dialect.
    type LexerConf: DialectLexerConf;

    /// The parser configuration of the dialect.
    type ParserConf: DialectParserConf;

    /// Returns the lexer configuration.
    fn lexer_conf(&self) -> &Self::LexerConf;

    /// Returns the parser configuration.
    fn parser_conf(&self) -> &Self::ParserConf;
}

/// The marker for a keyword definition.
pub trait KeywordDef
where
    Self: Clone + Debug + Display + 'static,
{
    /// All sorted keywords for the definition.
    const KEYWORDS: &'static [Self];

    /// All sorted keyword strings for the definition.
    const KEYWORD_STRINGS: &'static [&'static str];
}

/// The configuration of the lexer part of dialect.
pub trait DialectLexerConf: Clone + Debug {
    /// Determine if a character is the quotation mark of string literal.
    /// The default implementation, "single quote" is the quotation mark of string literal
    /// (both ANSI-compliant and most dialects, except MySQL).
    fn is_string_literal_quotation(&self, ch: char) -> bool {
        ch == '\''
    }

    /// Determine if a character starts a quoted identifier.
    /// The default implementation, accepting "double quoted" ids is both ANSI-compliant and
    /// appropriate for most dialects (with the notable exception of MySQL, and SQLite).
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }

    /// Determine if a character is a valid start character for an unquoted identifier.
    /// The default implementation is ANSI SQL.
    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic()
    }

    /// Determine if a character is a valid part character for an unquoted identifier.
    /// The default implementation is ANSI SQL.
    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    /// Determine if the whitespace token will be ignored.
    /// If enable it, whitespace will be excluded from the result of lexical analysis.
    fn ignore_whitespace(&self) -> bool {
        false
    }

    /// Determine if the comment token will be ignored.
    /// If enable it, comment will be excluded from the result of lexical analysis.
    fn ignore_comment(&self) -> bool {
        false
    }
}

/// The configuration of the parser part of dialect.
pub trait DialectParserConf: Clone + Debug {}
