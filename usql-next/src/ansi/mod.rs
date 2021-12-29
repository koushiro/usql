mod keyword;

pub use self::keyword::AnsiKeyword;
use crate::dialect::{CustomDialect, DialectLexerConf, DialectParserConf};

/// The ANSI dialect.
pub type AnsiDialect = CustomDialect<AnsiKeyword, AnsiLexerConfig, AnsiParserConfig>;

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
