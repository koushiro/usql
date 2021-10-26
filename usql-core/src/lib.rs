//! # usql-core
//!
//! usql-core is a core library includes some types and traits for usql.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod dialect;
#[macro_use]
mod keyword;

/// ANSI SQL-2016.
mod ansi;
/// MySQL.
#[cfg(feature = "mysql")]
mod mysql;
/// PostgreSQL.
#[cfg(feature = "postgres")]
mod postgres;
/// SQLite.
#[cfg(feature = "sqlite")]
mod sqlite;

pub use self::{dialect::Dialect, keyword::KeywordDef};
