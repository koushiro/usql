mod ddl;
mod dml;
mod transaction;

use core::fmt;

pub use self::{ddl::*, dml::*, transaction::*};

/// A top-level statement (SELECT, INSERT, CREATE, etc.)
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stmt {
    // ========================================================================
    // Data definition
    // ========================================================================
    /// The `CREATE SCHEMA ...` statement
    CreateSchema(CreateSchemaStmt),

    /// The `CREATE TABLE ...` statement
    CreateTable(CreateTableStmt),
    /// The `ALTER TABLE ...` statement
    AlterTable(AlterTableStmt),

    /// The `CREATE VIEW ...` statement
    CreateView(CreateViewStmt),

    /// The `CREATE DOMAIN ...` statement
    CreateDomain(CreateDomainStmt),
    /// The `ALTER DOMAIN ...` statement
    AlterDomain(AlterDomainStmt),

    /// The `CREATE TYPE ...` statement
    CreateType(CreateTypeStmt),
    /// The `ALTER TYPE ...` statement
    AlterType(AlterTypeStmt),

    /// The `CREATE DATABASE ...` statement (Not ANSI SQL standard)
    ///
    /// **NOTE**: not part of the ANSI SQL standard, and thus its syntax varies among vendors.
    CreateDatabase(CreateDatabaseStmt),
    /// The `CREATE INDEX ...` statement (Not ANSI SQL standard)
    ///
    /// **NOTE**: not part of the ANSI SQL standard, and thus its syntax varies among vendors.
    CreateIndex(CreateIndexStmt),

    /// The `DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | DATABASE | INDEX } ...` statement
    Drop(DropStmt),

    // ========================================================================
    // Data manipulation
    // ========================================================================
    /// The `INSERT INTO ...` statement
    Insert(InsertStmt),
    /// The `DELETE FROM ...` statement
    Delete(DeleteStmt),
    /// The `UPDATE ... SET ...` statement
    Update(UpdateStmt),
    /// The `SELECT ...` statement
    Select(SelectStmt),

    // ========================================================================
    // Transaction management
    // ========================================================================
    /// The `START TRANSACTION ...` statement
    StartTransaction(StartTransactionStmt),
    /// The `SET TRANSACTION ...` statement
    SetTransaction(SetTransactionStmt),
    /// The `COMMIT ...` statement
    CommitTransaction(CommitTransactionStmt),
    /// The `ROLLBACK ...` statement
    RollbackTransaction(RollbackTransactionStmt),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateSchema(stmt) => write!(f, "{}", stmt),
            Self::CreateTable(stmt) => write!(f, "{}", stmt),
            Self::AlterTable(stmt) => write!(f, "{}", stmt),
            Self::CreateView(stmt) => write!(f, "{}", stmt),
            Self::CreateDomain(stmt) => write!(f, "{}", stmt),
            Self::AlterDomain(stmt) => write!(f, "{}", stmt),
            Self::CreateType(stmt) => write!(f, "{}", stmt),
            Self::AlterType(stmt) => write!(f, "{}", stmt),
            Self::CreateDatabase(stmt) => write!(f, "{}", stmt),
            Self::CreateIndex(stmt) => write!(f, "{}", stmt),
            Self::Drop(stmt) => write!(f, "{}", stmt),

            Self::Insert(stmt) => write!(f, "{}", stmt),
            Self::Delete(stmt) => write!(f, "{}", stmt),
            Self::Update(stmt) => write!(f, "{}", stmt),
            Self::Select(stmt) => write!(f, "{}", stmt),

            Self::StartTransaction(stmt) => write!(f, "{}", stmt),
            Self::SetTransaction(stmt) => write!(f, "{}", stmt),
            Self::CommitTransaction(stmt) => write!(f, "{}", stmt),
            Self::RollbackTransaction(stmt) => write!(f, "{}", stmt),
        }
    }
}
