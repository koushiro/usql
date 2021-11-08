//! # usql-parser
//!
//! usql-parser is a universal SQL parser, which converts a sequence of tokens into abstract syntax tree.

#![deny(missing_docs, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod error;
mod parser;

pub use self::{error::ParserError, parser::*};
