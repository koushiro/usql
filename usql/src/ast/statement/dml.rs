#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::ast::{expression::*, types::*, utils::display_comma_separated};

/// The `INSERT INTO ...` statement.
///
/// ```txt
/// <insert statement> ::= INERT INTO <table name> <insert columns and source>
///
/// 1. INSERT INTO <table name> [ (column1, column2, ...) ]
///     [ OVERRIDING { SYSTEM | USER } VALUE ] <query expression>
/// 2. INSERT INTO <table name> [ (column1, column2, ...) ]
///     [ OVERRIDING { SYSTEM | USER } VALUE ] VALUES (value1, value2, ...)
/// 3. INSERT INTO <table name> DEFAULT VALUES
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsertStmt {
    /// Table name.
    pub table: ObjectName,
    /// Columns and source.
    pub source: InsertSource,
}

impl fmt::Display for InsertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INSERT INTO {}", self.table)?;
        write!(f, " {}", self.source)
    }
}

/// The source of insertion.
///
/// ```txt
/// <insert columns and source> ::= <from subquery> | <from constructor> | <from default>
///
/// <default> ::= DEFAULT VALUES
/// <from constructor> ::=
///     [ ( <column name> [, ...] ) ]
///     [ OVERRIDING { SYSTEM | USER } VALUE ]
///     VALUES <row value expression> [, ...]
/// <from subquery> ::=
///     [ ( <column name> [, ...] ) ]
///     [ OVERRIDING { SYSTEM | USER } VALUE ]
///     <query expression>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InsertSource {
    /// From default
    Default,
    /// From constructor
    Values {
        /// Column list.
        columns: Option<Vec<Ident>>,
        /// Overriding clause.
        overriding: Option<InsertOverriding>,
        /// Values.
        values: Values,
    },
    /// From subquery
    Subquery {
        /// Column list.
        columns: Option<Vec<Ident>>,
        /// Overriding clause.
        overriding: Option<InsertOverriding>,
        /// Subquery.
        subquery: Box<Query>,
    },
}

impl fmt::Display for InsertSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subquery {
                columns,
                overriding,
                subquery,
            } => {
                if let Some(columns) = columns {
                    write!(f, "({})", display_comma_separated(columns))?;
                }
                if let Some(overriding) = overriding {
                    write!(f, " {}", overriding)?;
                }
                write!(f, " {}", subquery)
            }
            Self::Values {
                columns,
                overriding,
                values,
            } => {
                if let Some(columns) = columns {
                    write!(f, "({})", display_comma_separated(columns))?;
                }
                if let Some(overriding) = overriding {
                    write!(f, " {}", overriding)?;
                }
                write!(f, " {}", values)
            }
            Self::Default => f.write_str("DEFAULT VALUES"),
        }
    }
}

/// The overriding clause of the `INSERT INTO ...` statement.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InsertOverriding {
    /// Overriding the system value
    System,
    /// Overriding the user value
    User,
}

impl fmt::Display for InsertOverriding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => f.write_str("OVERRIDING SYSTEM VALUE"),
            Self::User => f.write_str("OVERRIDING USER VALUE"),
        }
    }
}

/// The `DELETE FROM ...` statement.
///
/// ```txt
/// DELETE FROM <table> [ WHERE <search condition> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeleteStmt {
    /// Table name.
    pub table: ObjectName,
    /// Table alias.
    pub alias: Option<Ident>,
    /// Search condition.
    pub selection: Option<Where>,
}

impl fmt::Display for DeleteStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DELETE FROM {}", self.table)?;
        if let Some(alias) = &self.alias {
            write!(f, " AS {}", alias)?;
        }
        if let Some(selection) = &self.selection {
            write!(f, " {}", selection)?;
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateStmt {
    /// Table name.
    pub table: ObjectName,
    /// Table alias.
    pub alias: Option<Ident>,
    /// Column assignments.
    pub assignments: Vec<Assignment>,
    /// Search condition.
    pub selection: Option<Where>,
}

impl fmt::Display for UpdateStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UPDATE {}", self.table)?;
        if let Some(alias) = &self.alias {
            write!(f, " AS {}", alias)?;
        }
        write!(f, " SET {}", display_comma_separated(&self.assignments))?;
        if let Some(selection) = &self.selection {
            write!(f, " {}", selection)?;
        }
        Ok(())
    }
}

/// SQL assignment `foo = expr` as used in `Update` statement.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Assignment {
    /// Set target.
    pub target: Ident,
    /// Update source.
    pub value: Box<Expr>,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}

/// The `SELECT ...` statement.
///
/// See query expression for details.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectStmt(pub Box<Query>);

impl fmt::Display for SelectStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
