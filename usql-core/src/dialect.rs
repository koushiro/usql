use core::{any::Any, fmt::Debug};

use crate::keyword::KeywordDef;

///
pub trait Dialect: Debug + Any {
    ///
    type Keyword: KeywordDef;

    ///
    type LexerConf: DialectLexerConf;

    ///
    type ParserConf: DialectParserConf;
}

///
pub trait DialectLexerConf: Debug {}

///
pub trait DialectParserConf: Debug {}
