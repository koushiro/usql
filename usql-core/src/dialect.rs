use core::{any::Any, fmt::Debug};

use crate::keyword::KeywordDef;

///
pub trait Dialect: Debug + Any {
    ///
    type Keyword: KeywordDef;
}
