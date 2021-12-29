#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::error::ParserError;

pub type ParseStream<'a> = &'a str;

///
pub trait Parse: Sized {
    ///
    fn parse(input: ParseStream) -> Result<Self, ParserError>;
}
