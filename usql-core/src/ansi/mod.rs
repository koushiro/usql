mod keyword;

pub use self::keyword::AnsiKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The ANSI dialect.
#[derive(Clone, Debug, Default)]
pub struct AnsiDialect {
    /// ANSI lexer configuration.
    pub lexer_conf: AnsiLexerConfig,
    /// ANSI parser configuration.
    pub parser_conf: AnsiParserConfig,
}

impl Dialect for AnsiDialect {
    type Keyword = AnsiKeyword;
    type LexerConf = AnsiLexerConfig;
    type ParserConf = AnsiParserConfig;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

/// The lexer configuration of ANSI dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnsiLexerConfig {}

impl DialectLexerConf for AnsiLexerConfig {}

/// The parser configuration of ANSI dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnsiParserConfig {}

impl DialectParserConf for AnsiParserConfig {}
