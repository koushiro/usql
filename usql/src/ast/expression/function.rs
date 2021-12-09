#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

use crate::ast::{
    expression::{Expr, WindowSpec},
    types::{Ident, ObjectName},
    utils::display_comma_separated,
};

/// A function call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Function {
    // aggregate functions may specify e.g. `COUNT(DISTINCT x)`
    #[doc(hidden)]
    pub distinct: bool,
    /// The name of the function.
    pub name: ObjectName,
    /// The arguments of the function.
    pub args: Vec<FunctionArg>,
    /// The over clause.
    pub over: Option<WindowSpec>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({}{})",
            self.name,
            if self.distinct { "DISTINCT " } else { "" },
            display_comma_separated(&self.args),
        )?;
        if let Some(o) = &self.over {
            write!(f, " OVER ({})", o)?;
        }
        Ok(())
    }
}

/// The arguments of a function call.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FunctionArg {
    /// Named argument.
    #[doc(hidden)]
    Named { name: Ident, arg: Expr },
    /// Unnamed argument.
    Unnamed(Expr),
}

impl fmt::Display for FunctionArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionArg::Named { name, arg } => write!(f, "{} => {}", name, arg),
            FunctionArg::Unnamed(unnamed_arg) => write!(f, "{}", unnamed_arg),
        }
    }
}
