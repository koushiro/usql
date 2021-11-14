mod keyword;

pub use self::keyword::AnsiKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The ANSI dialect.
#[derive(Debug)]
pub struct AnsiDialect;

/// The lexer configuration of ANSI dialect.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnsiLexerConfig {}

impl DialectLexerConf for AnsiLexerConfig {}

/// The parser configuration of ANSI dialect.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnsiParserConfig {}

impl DialectParserConf for AnsiParserConfig {}

impl Dialect for AnsiDialect {
    type Keyword = AnsiKeyword;
    type LexerConf = AnsiLexerConfig;
    type ParserConf = AnsiParserConfig;
}
