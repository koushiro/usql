#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{expression::*, types::*, utils::display_comma_separated};

/// The `INSERT INTO ...` statement.
///
/// ```txt
/// INSERT INTO <table name> [ (column1, column2, ...) ] [SELECT ...]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InsertStmt {
    /// Table name.
    pub table: ObjectName,
    /// Column list.
    pub columns: Vec<Ident>,
    /// A SQL query that specifies what to insert.
    pub source: Option<Box<Query>>,
}

impl fmt::Display for InsertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INSERT INTO {}", self.table)?;
        if !self.columns.is_empty() {
            write!(f, "({})", display_comma_separated(&self.columns))?;
        }
        if let Some(source) = &self.source {
            write!(f, "{}", source)?;
        }
        Ok(())
    }
}

/// The `DELETE FROM ...` statement.
///
/// ```txt
/// DELETE FROM <table> [ WHERE <search condition> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeleteStmt {
    /// Table name.
    pub table: ObjectName,
    /// Search condition.
    pub selection: Option<Expr>,
}

impl fmt::Display for DeleteStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DELETE FROM {}", self.table)?;
        if let Some(selection) = &self.selection {
            write!(f, "WHERE {}", selection)?;
        }
        Ok(())
    }
}

/// The `UPDATE ... SET ...` statement.
///
/// ```txt
/// UPDATE <table> SET <assignments> [ WHERE <search condition> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdateStmt {
    /// Table name.
    pub table: ObjectName,
    /// Column assignments.
    pub assignments: Vec<Assignment>,
    /// Search condition.
    pub selection: Option<Expr>,
}

impl fmt::Display for UpdateStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UPDATE {}", self.table)?;
        if !self.assignments.is_empty() {
            write!(f, " SET {}", display_comma_separated(&self.assignments))?;
        }
        if let Some(selection) = &self.selection {
            write!(f, "WHERE {}", selection)?;
        }
        Ok(())
    }
}

/// SQL assignment `foo = expr` as used in `Update` statement.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Assignment {
    /// Set target.
    pub target: Ident,
    /// Update source.
    pub value: Expr,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}

/// The `SELECT ...` statement.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelectStmt(pub Box<Query>);

impl fmt::Display for SelectStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
