mod keyword;

pub use self::keyword::PostgresKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The PostgreSQL dialect.
#[derive(Debug)]
pub struct PostgresDialect;

/// The lexer configuration of PostgreSQL dialect.
#[derive(Debug)]
pub struct PostgresLexerConfig {}

impl DialectLexerConf for PostgresLexerConfig {}

/// The parser configuration of PostgreSQL dialect.
#[derive(Debug)]
pub struct PostgresParserConfig {}

impl DialectParserConf for PostgresParserConfig {}

impl Dialect for PostgresDialect {
    type Keyword = PostgresKeyword;
    type LexerConf = PostgresLexerConfig;
    type ParserConf = PostgresParserConfig;
}
