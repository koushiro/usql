//! # uSQL
//!
//! uSQL is a universal SQL lexer and parser library.

#![warn(missing_docs)]
#![warn(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;

// pub mod ast;
mod dialect;
mod error;
mod lexer;
mod span;
mod token;

pub use self::{
    dialect::{CustomDialect, Dialect, DialectLexerConf, DialectParserConf},
    error::{LexerError, ParserError},
    lexer::Lexer,
    span::{Span, Spanned},
    token::{Keyword, KeywordDef},
};
