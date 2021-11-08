use core::fmt;

/// Parser error
#[derive(Clone, Debug, PartialEq)]
pub enum ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParserError {}
