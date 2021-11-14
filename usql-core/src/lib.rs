//! # usql-core
//!
//! usql-core is a core library includes some types and traits for usql.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
mod macros;
mod dialect;
mod tokens;

/// ANSI SQL-2016.
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
    dialect::{Dialect, DialectLexerConf, DialectParserConf, KeywordDef},
    tokens::{Comment, Ident, Token, Whitespace},
};
