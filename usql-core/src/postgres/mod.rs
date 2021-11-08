mod keyword;

pub use self::keyword::PostgresKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

///
#[derive(Debug)]
pub struct PostgresDialect;

///
#[derive(Debug)]
pub struct PostgresLexerConfig {}

impl DialectLexerConf for PostgresLexerConfig {}

///
#[derive(Debug)]
pub struct PostgresParserConfig {}

impl DialectParserConf for PostgresParserConfig {}

impl Dialect for PostgresDialect {
    type Keyword = PostgresKeyword;
    type LexerConf = PostgresLexerConfig;
    type ParserConf = PostgresParserConfig;
}
