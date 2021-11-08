mod keyword;

pub use self::keyword::SqliteKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The Sqlite dialect.
#[derive(Debug)]
pub struct SqliteDialect;

/// The lexer configuration of Sqlite dialect.
#[derive(Debug)]
pub struct SqliteLexerConfig {}

impl DialectLexerConf for SqliteLexerConfig {}

/// The parser configuration of Sqlite dialect.
#[derive(Debug)]
pub struct SqliteParserConfig {}

impl DialectParserConf for SqliteParserConfig {}

impl Dialect for SqliteDialect {
    type Keyword = SqliteKeyword;
    type LexerConf = SqliteLexerConfig;
    type ParserConf = SqliteParserConfig;
}
