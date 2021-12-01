#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

use crate::utils::display_comma_separated;

/// The `START TRANSACTION ...` statement.
///
/// ```txt
/// { START TRANSACTION | BEGIN [ TRANSACTION | WORK ] } [ <mode>, ... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartTransactionStmt {
    /// The transaction characteristics.
    pub characteristics: Vec<TransactionCharacteristic>,
}

impl fmt::Display for StartTransactionStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("START TRANSACTION")?;
        if !self.characteristics.is_empty() {
            write!(f, " {}", display_comma_separated(&self.characteristics))?;
        }
        Ok(())
    }
}

/// The `SET TRANSACTION ...` statement.
///
/// ```txt
/// SET TRANSACTION [ <mode>, ... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetTransactionStmt {
    /// The transaction characteristics.
    pub characteristics: Vec<TransactionCharacteristic>,
}

impl fmt::Display for SetTransactionStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SET TRANSACTION")?;
        if !self.characteristics.is_empty() {
            write!(f, " {}", display_comma_separated(&self.characteristics))?;
        }
        Ok(())
    }
}

/// The transaction characteristic.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionCharacteristic {
    AccessMode(TransactionAccessMode),
    IsolationLevel(TransactionIsolationLevel),
}

impl fmt::Display for TransactionCharacteristic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccessMode(mode) => write!(f, "{}", mode),
            Self::IsolationLevel(level) => write!(f, "ISOLATION LEVEL {}", level),
        }
    }
}

impl From<TransactionAccessMode> for TransactionCharacteristic {
    fn from(mode: TransactionAccessMode) -> Self {
        Self::AccessMode(mode)
    }
}

impl From<TransactionIsolationLevel> for TransactionCharacteristic {
    fn from(level: TransactionIsolationLevel) -> Self {
        Self::IsolationLevel(level)
    }
}

/// The access mode of transaction.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionAccessMode {
    ReadOnly,
    ReadWrite,
}

impl fmt::Display for TransactionAccessMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::ReadOnly => "READ ONLY",
            Self::ReadWrite => "READ WRITE",
        })
    }
}

/// The isolation level of transaction.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionIsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl fmt::Display for TransactionIsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        })
    }
}

/// The `COMMIT ...` statement.
///
/// ```txt
/// COMMIT [ TRANSACTION | WORK ] [ AND [ NO ] CHAIN ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommitTransactionStmt {
    /// Flag to indicate whether a new transaction is immediately started with
    /// the same transaction characteristics as the just finished one.
    pub and_chain: bool,
}

impl fmt::Display for CommitTransactionStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "COMMIT{}",
            if self.and_chain { " AND CHAIN" } else { "" }
        )
    }
}

/// The `ROLLBACK ...` statement.
///
/// ```txt
/// ROLLBACK [ TRANSACTION | WORK ] [ AND [ NO ] CHAIN ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RollbackTransactionStmt {
    /// Flag to indicate whether a new transaction is immediately started with
    /// the same transaction characteristics as the just finished one.
    pub and_chain: bool,
}

impl fmt::Display for RollbackTransactionStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ROLLBACK{}",
            if self.and_chain { " AND CHAIN" } else { "" }
        )
    }
}
