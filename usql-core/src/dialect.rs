use core::{
    any::Any,
    fmt::{Debug, Display},
};

/// The marker for a dialect.
pub trait Dialect: Debug + Any {
    /// The keyword definition of the dialect.
    type Keyword: KeywordDef;

    /// The lexer config of the dialect.
    type LexerConf: DialectLexerConf;

    /// The parser config of the dialect.
    type ParserConf: DialectParserConf;
}

/// The marker for a keyword definition.
pub trait KeywordDef
where
    Self: Clone + Display + 'static,
{
    /// All sorted keywords for the definition.
    const KEYWORDS: &'static [Self];

    /// All sorted keyword strings for the definition.
    const KEYWORD_STRINGS: &'static [&'static str];
}

/// The configuration of the lexer part of dialect.
pub trait DialectLexerConf: Debug {}

/// The configuration of the parser part of dialect.
pub trait DialectParserConf: Debug {}
