mod keyword;

pub use self::keyword::SqliteKeyword;
use crate::dialect::Dialect;

#[derive(Debug)]
pub struct SqliteDialect;

impl Dialect for SqliteDialect {
    type Keyword = SqliteKeyword;
}
