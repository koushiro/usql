mod keyword;
mod punct;

pub use self::{
    keyword::{Keyword, KeywordDef},
    punct::*,
};

/// Marker trait for types that represent single token.
pub trait Token {
    fn peek() -> bool;
}

#[doc(hidden)]
pub trait CustomToken {
    fn peek() -> bool;
}

impl<T: CustomToken> Token for T {
    fn peek() -> bool {
        <Self as CustomToken>::peek()
    }
}
