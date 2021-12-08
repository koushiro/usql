//! # usql
//!
//! uSQL is a universal SQL lexer and parser.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;
mod dialect;
mod keywords;

/// Universal SQL AST types.
pub mod ast;
/// Universal SQL lexer.
pub mod lexer;
/// Universal SQL parser.
pub mod parser;

/// ANSI SQL-2016.
#[cfg(feature = "ansi")]
pub mod ansi;
/// MySQL 8.0.
#[cfg(feature = "mysql")]
pub mod mysql;
/// PostgreSQL 13.
#[cfg(feature = "postgres")]
pub mod postgres;
/// SQLite 3.
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use self::{
    dialect::{CustomDialect, Dialect, DialectLexerConf, DialectParserConf},
    keywords::{Keyword, KeywordDef},
};
