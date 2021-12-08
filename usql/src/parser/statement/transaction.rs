#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::{
    ast::statement::*,
    dialect::Dialect,
    keywords::Keyword,
    lexer::Token,
    parser::{error::ParserError, Parser},
};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parses a `START TRANSACTION` statement.
    pub fn parse_start_transaction_stmt(&mut self) -> Result<StartTransactionStmt, ParserError> {
        // ANSI/MySQL/PostgreSQL: START TRANSACTION [ <transaction characteristic> [, ...] ]
        // SQLite: not supported, need to use `BEGIN TRANSACTION` instead
        self.expect_keywords(&[Keyword::START, Keyword::TRANSACTION])?;
        Ok(StartTransactionStmt {
            characteristics: self.parse_transaction_characteristics(false)?,
        })
    }

    /// Parses a `BEGIN` statement.
    ///
    /// `BEGIN` is a nonstandard but common alias for the standard `START TRANSACTION` statement.
    /// It is supported by at least MySQL, PostgreSQL and SQLite.
    pub fn parse_begin_stmt(&mut self) -> Result<StartTransactionStmt, ParserError> {
        // MySQL: BEGIN [WORK]
        // PostgreSQL: BEGIN [ WORK | TRANSACTION ] [ <transaction characteristic> [, ...] ]
        // SQLite: BEGIN [ DEFERRED | IMMEDIATE | EXCLUSIVE ] [ TRANSACTION ]
        self.expect_keyword(Keyword::BEGIN)?;
        let _ = self.parse_one_of_keywords(&[
            Keyword::DEFERRED,
            Keyword::IMMEDIATE,
            Keyword::EXCLUSIVE,
        ]);
        let _ = self.parse_one_of_keywords(&[Keyword::WORK, Keyword::TRANSACTION]);
        Ok(StartTransactionStmt {
            characteristics: self.parse_transaction_characteristics(false)?,
        })
    }

    /// Parses a `SET TRANSACTION` statement.
    pub fn parse_set_transaction_stmt(&mut self) -> Result<SetTransactionStmt, ParserError> {
        // ANSI: SET [ LOCAL ] TRANSACTION  <transaction characteristic> [, ...]
        // MySQL: SET [GLOBAL | SESSION] TRANSACTION <transaction characteristic> [, ...]
        // PostgreSQL: SET TRANSACTION <transaction characteristic> [, ...]
        // SQLite: not supported
        self.expect_keyword(Keyword::SET)?;
        let _ = self.parse_one_of_keywords(&[Keyword::LOCAL, Keyword::GLOBAL, Keyword::SESSION]);
        self.expect_keyword(Keyword::TRANSACTION)?;
        Ok(SetTransactionStmt {
            characteristics: self.parse_transaction_characteristics(true)?,
        })
    }

    /// Parses transaction characteristics.
    pub fn parse_transaction_characteristics(
        &mut self,
        mut required: bool,
    ) -> Result<Vec<TransactionCharacteristic>, ParserError> {
        let mut characteristics = vec![];
        loop {
            let characteristic = if self.parse_keywords(&[Keyword::ISOLATION, Keyword::LEVEL]) {
                let iso_level = if self.parse_keywords(&[Keyword::READ, Keyword::UNCOMMITTED]) {
                    TransactionIsolationLevel::ReadUncommitted
                } else if self.parse_keywords(&[Keyword::READ, Keyword::COMMITTED]) {
                    TransactionIsolationLevel::ReadCommitted
                } else if self.parse_keywords(&[Keyword::REPEATABLE, Keyword::READ]) {
                    TransactionIsolationLevel::RepeatableRead
                } else if self.parse_keyword(Keyword::SERIALIZABLE) {
                    TransactionIsolationLevel::Serializable
                } else {
                    let found = self.peek_token().cloned();
                    self.expected("isolation level", found)?
                };
                TransactionCharacteristic::IsolationLevel(iso_level)
            } else if self.parse_keywords(&[Keyword::READ, Keyword::ONLY]) {
                TransactionCharacteristic::AccessMode(TransactionAccessMode::ReadOnly)
            } else if self.parse_keywords(&[Keyword::READ, Keyword::WRITE]) {
                TransactionCharacteristic::AccessMode(TransactionAccessMode::ReadWrite)
            } else if required {
                let found = self.peek_token().cloned();
                self.expected("transaction characteristic", found)?
            } else {
                break;
            };
            characteristics.push(characteristic);
            required = self.next_token_if_is(&Token::Comma);
        }
        Ok(characteristics)
    }

    /// Parses a `COMMIT` statement.
    pub fn parse_commit_stmt(&mut self) -> Result<CommitTransactionStmt, ParserError> {
        // ANSI: COMMIT [ WORK ] [ AND [ NO ] CHAIN ]
        // MySQL: COMMIT [ WORK ] [ AND [ NO ] CHAIN ] [ [ NO ] RELEASE ]
        // PostgreSQL: { COMMIT | END } [ WORK | TRANSACTION ] [ AND [ NO ] CHAIN ]
        // SQLite: { COMMIT | END } [ TRANSACTION ]
        self.expect_keyword(Keyword::COMMIT)?;
        let _ = self.parse_one_of_keywords(&[Keyword::TRANSACTION, Keyword::WORK]);
        let and_chain = self.parse_commit_rollback_chain()?;
        Ok(CommitTransactionStmt { and_chain })
    }

    /// Parses a `ROLLBACK` statement.
    pub fn parse_rollback_stmt(&mut self) -> Result<RollbackTransactionStmt, ParserError> {
        // ANSI: ROLLBACK [ WORK ] [ AND [ NO ] CHAIN ] [ TO SAVEPOINT <savepoint specifier> ]
        // MySQL: ROLLBACK [ WORK ] [ AND [ NO ] CHAIN ] [ [ NO ] RELEASE ]
        // PostgreSQL: ROLLBACK [ WORK | TRANSACTION ] [ AND [ NO ] CHAIN ]
        // SQLite: ROLLBACK [ TRANSACTION ]  [ TO [ SAVEPOINT ] <savepoint name> ]
        self.expect_keyword(Keyword::ROLLBACK)?;
        let _ = self.parse_one_of_keywords(&[Keyword::TRANSACTION, Keyword::WORK]);
        let and_chain = self.parse_commit_rollback_chain()?;
        Ok(RollbackTransactionStmt { and_chain })
    }

    fn parse_commit_rollback_chain(&mut self) -> Result<bool, ParserError> {
        if self.parse_keyword(Keyword::AND) {
            let chain = !self.parse_keyword(Keyword::NO);
            self.expect_keyword(Keyword::CHAIN)?;
            Ok(chain)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::error::parse_error;

    #[test]
    fn parse_start_transaction_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "START TRANSACTION ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_start_transaction_stmt()?,
            StartTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        let sql = "START TRANSACTION READ WRITE, ISOLATION LEVEL READ UNCOMMITTED";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_start_transaction_stmt()?,
            StartTransactionStmt {
                characteristics: vec![
                    TransactionAccessMode::ReadWrite.into(),
                    TransactionIsolationLevel::ReadUncommitted.into()
                ]
            }
        );
        let sql = "START TRANSACTION";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_start_transaction_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_begin_stmt() -> Result<(), ParserError> {
        let dialect = crate::mysql::MysqlDialect::default();
        // MySQL: BEGIN [ WORK ]
        assert_eq!(
            Parser::new_with_sql(&dialect, "BEGIN")?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "BEGIN WORK")?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        // PostgreSQL: BEGIN [ WORK | TRANSACTION ] [ transaction_mode [, ...] ]
        let dialect = crate::postgres::PostgresDialect::default();
        let sql = "BEGIN TRANSACTION ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        let sql = "BEGIN WORK ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        // SQLite: BEGIN [ DEFERRED | IMMEDIATE | EXCLUSIVE ] [ TRANSACTION ]
        let dialect = crate::sqlite::SqliteDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "BEGIN")?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "BEGIN TRANSACTION")?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "BEGIN IMMEDIATE TRANSACTION")?.parse_begin_stmt()?,
            StartTransactionStmt {
                characteristics: vec![]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_set_transaction_stmt() -> Result<(), ParserError> {
        // ANSI: SET [ LOCAL ] TRANSACTION  transaction_mode [, ...]
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "SET LOCAL TRANSACTION ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_set_transaction_stmt()?,
            SetTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "SET TRANSACTION")?.parse_set_transaction_stmt(),
            parse_error("Expected: transaction characteristic, but not found")
        );
        // MySQL: SET [GLOBAL | SESSION] TRANSACTION transaction_mode [, ...]
        let dialect = crate::mysql::MysqlDialect::default();
        let sql = "SET SESSION TRANSACTION ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_set_transaction_stmt()?,
            SetTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        // PostgreSQL: SET TRANSACTION transaction_mode [, ...]
        let dialect = crate::postgres::PostgresDialect::default();
        let sql = "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED, READ ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_set_transaction_stmt()?,
            SetTransactionStmt {
                characteristics: vec![
                    TransactionIsolationLevel::ReadUncommitted.into(),
                    TransactionAccessMode::ReadOnly.into()
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_commit_stmt() -> Result<(), ParserError> {
        // ANSI: COMMIT [ WORK ] [ AND [ NO ] CHAIN ]
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT WORK")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT AND CHAIN")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT AND NO CHAIN")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        // MySQL: COMMIT [ WORK ] [ AND [ NO ] CHAIN ] [ [ NO ] RELEASE ]
        let dialect = crate::mysql::MysqlDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT WORK")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT AND CHAIN")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT AND NO CHAIN")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        // PostgreSQL: COMMIT [ WORK | TRANSACTION ] [ AND [ NO ] CHAIN ]
        let dialect = crate::postgres::PostgresDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT WORK")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT TRANSACTION")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT TRANSACTION AND CHAIN")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT TRANSACTION AND NO CHAIN")?
                .parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        // SQLite: COMMIT [ TRANSACTION ]
        let dialect = crate::sqlite::SqliteDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "COMMIT TRANSACTION")?.parse_commit_stmt()?,
            CommitTransactionStmt { and_chain: false }
        );
        Ok(())
    }

    #[test]
    fn parse_rollback_stmt() -> Result<(), ParserError> {
        // ANSI: ROLLBACK [ WORK ] [ AND [ NO ] CHAIN ] [ TO SAVEPOINT <savepoint specifier> ]
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND NO CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        // MySQL: ROLLBACK [ WORK ] [ AND [ NO ] CHAIN ] [ [ NO ] RELEASE ]
        let dialect = crate::mysql::MysqlDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND NO CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        // PostgreSQL: ROLLBACK [ WORK | TRANSACTION ] [ AND [ NO ] CHAIN ]
        let dialect = crate::postgres::PostgresDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK TRANSACTION")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: true }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK WORK AND NO CHAIN")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        // SQLite: ROLLBACK [ TRANSACTION ]  [ TO [ SAVEPOINT ] <savepoint name> ]
        let dialect = crate::sqlite::SqliteDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ROLLBACK TRANSACTION")?.parse_rollback_stmt()?,
            RollbackTransactionStmt { and_chain: false }
        );
        Ok(())
    }
}
