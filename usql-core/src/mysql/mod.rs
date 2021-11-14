mod keyword;

pub use self::keyword::MysqlKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The MySQL dialect.
#[derive(Debug)]
pub struct MysqlDialect;

/// The lexer configuration of MySQL dialect.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MysqlLexerConfig {}

impl DialectLexerConf for MysqlLexerConfig {}

/// The parser configuration of MySQL dialect.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MysqlParserConfig {}

impl DialectParserConf for MysqlParserConfig {}

impl Dialect for MysqlDialect {
    type Keyword = MysqlKeyword;
    type LexerConf = MysqlLexerConfig;
    type ParserConf = MysqlParserConfig;
}
