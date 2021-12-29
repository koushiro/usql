//! # usql
//!
//! uSQL is a universal SQL Lexer and Parser.

#![warn(missing_docs)]
#![warn(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;

mod dialect;
mod error;
// mod parse;
mod token;

mod ansi;

mod ast;
mod lexer;
mod span;

pub use self::{
    ansi::AnsiDialect,
    error::{LexerError, ParserError},
    lexer::Lexer,
    span::{Span, Spanned},
    token::{Keyword, KeywordDef},
};
