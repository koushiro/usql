mod keyword;

pub use self::keyword::PostgresKeyword;
use crate::dialect::Dialect;

#[derive(Debug)]
pub struct PostgresDialect;

impl Dialect for PostgresDialect {
    type Keyword = PostgresKeyword;
}
