mod transaction;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use self::transaction::*;
use crate::expression::Query;

/// A top-level statement (SELECT, INSERT, CREATE, etc.)
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Stmt {
    // ========================================================================
    CreateDatabase(CreateDatabaseStmt),
    CreateTable(CreateTableStmt),
    CreateIndex(CreateIndexStmt),
    CreateView(CreateViewStmt),
    CreateSchema(CreateSchemaStmt),

    AlterTable(AlterTableStmt),

    Drop(DropStmt),

    // /// EXPLAIN / DESCRIBE for select_statement
    // Explain {
    //     // If true, query used the MySQL `DESCRIBE` alias for explain
    //     describe_alias: bool,
    //     /// Carry out the command and show actual run times and other statistics.
    //     analyze: bool,
    //     // Display additional information regarding the plan.
    //     verbose: bool,
    //     /// A SQL query that specifies what to explain
    //     statement: Box<Statement>,
    // },

    // ========================================================================
    Insert(InsertStmt),

    Delete(DeleteStmt),

    Update(UpdateStmt),

    Query(Box<Query>),

    // ========================================================================
    /// `{ BEGIN [ TRANSACTION | WORK ] | START TRANSACTION } ...`
    StartTransaction {
        modes: Vec<TransactionMode>,
    },
    /// `SET TRANSACTION ...`
    SetTransaction {
        modes: Vec<TransactionMode>,
    },
    /// `COMMIT [ TRANSACTION | WORK ] [ AND [ NO ] CHAIN ]`
    Commit {
        chain: bool,
    },
    /// `ROLLBACK [ TRANSACTION | WORK ] [ AND [ NO ] CHAIN ]`
    Rollback {
        chain: bool,
    },
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

// ============================================================================
// Data Definition
// ============================================================================

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateDatabaseStmt {}

impl fmt::Display for CreateDatabaseStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateTableStmt {}

impl fmt::Display for CreateTableStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateIndexStmt {}

impl fmt::Display for CreateIndexStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateViewStmt {}

impl fmt::Display for CreateViewStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateSchemaStmt {}

impl fmt::Display for CreateSchemaStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AlterTableStmt {}

impl fmt::Display for AlterTableStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DropStmt {}

impl fmt::Display for DropStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

// ============================================================================
// Data Manipulation
// ============================================================================

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InsertStmt {}

impl fmt::Display for InsertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeleteStmt {}

impl fmt::Display for DeleteStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdateStmt {}

impl fmt::Display for UpdateStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
