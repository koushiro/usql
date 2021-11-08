mod keyword;

pub use self::keyword::MysqlKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

///
#[derive(Debug)]
pub struct MysqlDialect;

///
#[derive(Debug)]
pub struct MysqlLexerConfig {}

impl DialectLexerConf for MysqlLexerConfig {}

///
#[derive(Debug)]
pub struct MysqlParserConfig {}

impl DialectParserConf for MysqlParserConfig {}

impl Dialect for MysqlDialect {
    type Keyword = MysqlKeyword;
    type LexerConf = MysqlLexerConfig;
    type ParserConf = MysqlParserConfig;
}
