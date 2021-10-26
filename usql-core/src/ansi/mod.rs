mod keyword;

pub use self::keyword::AnsiKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

///
#[derive(Debug)]
pub struct AnsiDialect;

///
#[derive(Debug)]
pub struct AnsiLexerConfig {}

impl DialectLexerConf for AnsiLexerConfig {}

///
#[derive(Debug)]
pub struct AnsiParserConfig {}

impl DialectParserConf for AnsiParserConfig {}

impl Dialect for AnsiDialect {
    type Keyword = AnsiKeyword;
    type LexerConf = AnsiLexerConfig;
    type ParserConf = AnsiParserConfig;
}
