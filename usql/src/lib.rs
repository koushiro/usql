//! # usql
//!
//! uSQL is a Universal SQL Lexer and Parser.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use usql_ast as ast;
pub use usql_core as core;
pub use usql_lexer as lexer;
