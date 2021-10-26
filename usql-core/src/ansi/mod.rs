mod keyword;

pub use self::keyword::AnsiKeyword;
use crate::dialect::Dialect;

#[derive(Debug)]
pub struct AnsiDialect;

impl Dialect for AnsiDialect {
    type Keyword = AnsiKeyword;
}
