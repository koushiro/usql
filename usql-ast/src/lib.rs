//! # usql-ast
//!
//! usql-ast is a universal SQL parser ast types.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod dialect;
mod utils;

pub use self::dialect::*;

/// SQL expressions.
pub mod expression;
/// SQL statements.
pub mod statement;
/// SQL types (Literal, DataType, Ident, etc).
pub mod types;
