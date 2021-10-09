//! # usql-lexer
//!
//! usql-lexer is a universal SQL lexer, which converts a string into a sequence of tokens.

#![deny(missing_docs, unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod error;
/// The keywords definition.
pub mod keywords;
mod lexer;
mod tokens;

pub use self::{
    error::LexerError,
    keywords::KeywordDef,
    lexer::Lexer,
    tokens::{Comment, Ident, Token, Whitespace},
};
