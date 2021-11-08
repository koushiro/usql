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
#[macro_use]
mod keyword;

/// ANSI SQL-2016.
pub mod ansi;
/// MySQL.
#[cfg(feature = "mysql")]
pub mod mysql;
/// PostgreSQL.
#[cfg(feature = "postgres")]
pub mod postgres;
/// SQLite.
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use self::{
    dialect::{Dialect, DialectLexerConf, DialectParserConf},
    keyword::KeywordDef,
};
