mod keyword;

pub use self::keyword::MysqlKeyword;
use crate::dialect::Dialect;

#[derive(Debug)]
pub struct MysqlDialect;

impl Dialect for MysqlDialect {
    type Keyword = MysqlKeyword;
}

