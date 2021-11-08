mod keyword;

pub use self::keyword::SqliteKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

///
#[derive(Debug)]
pub struct SqliteDialect;

///
#[derive(Debug)]
pub struct SqliteLexerConfig {}

impl DialectLexerConf for SqliteLexerConfig {}

///
#[derive(Debug)]
pub struct SqliteParserConfig {}

impl DialectParserConf for SqliteParserConfig {}

impl Dialect for SqliteDialect {
    type Keyword = SqliteKeyword;
    type LexerConf = SqliteLexerConfig;
    type ParserConf = SqliteParserConfig;
}
