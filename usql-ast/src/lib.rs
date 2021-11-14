//! # usql-ast
//!
//! usql-ast is a universal SQL parser ast types.

#![deny(missing_docs)]
#![deny(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod dialect;
mod expression;
mod statement;
mod types;
mod utils;

pub use self::{dialect::*, expression::*, statement::*, types::*};
