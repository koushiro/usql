#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

use usql_ast::statement::*;
use usql_core::{Dialect, Keyword};

use crate::{error::ParserError, parser::Parser};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parses a `CREATE SCHEMA` statement.
    pub fn parse_create_schema_stmt(&mut self) -> Result<CreateSchemaStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::SCHEMA])?;
        let _if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);
        todo!()
    }

    /// Parses a `CREATE TABLE` statement.
    pub fn parse_create_table_stmt(&mut self) -> Result<CreateTableStmt, ParserError> {
        todo!()
    }

    /// Parses a column definition.
    fn parse_column_def(&mut self) -> Result<ColumnDef, ParserError> {
        // let constraints = self.parse_comma_separated(Self::parse_column_constraint_def)?;
        todo!()
    }

    /// Parses a column constraint definition.
    fn parse_column_constraint_def(&mut self) -> Result<ColumnConstraintDef, ParserError> {
        let name = if self.parse_keyword(Keyword::CONSTRAINT) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        let constraint = self.parse_column_constraint()?;
        Ok(ColumnConstraintDef { name, constraint })
    }

    /// Parses a column constraint.
    fn parse_column_constraint(&mut self) -> Result<ColumnConstraint, ParserError> {
        if self.parse_keyword(Keyword::NULL) {
            Ok(ColumnConstraint::Null)
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::NULL]) {
            Ok(ColumnConstraint::NotNull)
        } else if self.parse_keyword(Keyword::UNIQUE) {
            Ok(ColumnConstraint::Unique { is_primary: false })
        } else if self.parse_keywords(&[Keyword::PRIMARY, Keyword::KEY]) {
            Ok(ColumnConstraint::Unique { is_primary: true })
        } else if self.parse_keyword(Keyword::DEFAULT) {
            let default = self.parse_literal()?;
            Ok(ColumnConstraint::Default(default))
        } else if self.parse_keyword(Keyword::COLLATE) {
            let collation = self.parse_object_name()?;
            Ok(ColumnConstraint::Collation(collation))
        } else if self.parse_keyword(Keyword::REFERENCES) {
            let table = self.parse_object_name()?;
            Ok(ColumnConstraint::References {
                table,
                referenced_columns: Vec::new(),
                match_type: None,
                on_delete: None,
                on_update: None,
            })
        } else {
            todo!()
        }
    }

    /// Parses a table constraint definition.
    fn parse_table_constraint_def(&mut self) -> Result<TableConstraintDef, ParserError> {
        let name = if self.parse_keyword(Keyword::CONSTRAINT) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        let constraint = self.parse_table_constraint()?;
        Ok(TableConstraintDef { name, constraint })
    }

    /// Parses a table constraint.
    fn parse_table_constraint(&mut self) -> Result<TableConstraint, ParserError> {
        todo!()
    }

    fn parse_referential_match_type(&mut self) -> Result<ReferentialMatchType, ParserError> {
        match self.parse_one_of_keywords(&[Keyword::FULL, Keyword::PARTIAL, Keyword::SIMPLE]) {
            Some(keyword) => match keyword {
                Keyword::FULL => Ok(ReferentialMatchType::Full),
                Keyword::PARTIAL => Ok(ReferentialMatchType::Partial),
                Keyword::SIMPLE => Ok(ReferentialMatchType::Simple),
                _ => unreachable!(),
            },
            None => {
                let found = self.peek_token().cloned();
                self.expected("FULL, PARTIAL or SIMPLE", found)
            }
        }
    }

    fn parse_referential_action(&mut self) -> Result<ReferentialAction, ParserError> {
        if self.parse_keyword(Keyword::CASCADE) {
            Ok(ReferentialAction::Cascade)
        } else if self.parse_keyword(Keyword::RESTRICT) {
            Ok(ReferentialAction::Restrict)
        } else if self.parse_keywords(&[Keyword::SET, Keyword::NULL]) {
            Ok(ReferentialAction::SetNull)
        } else if self.parse_keywords(&[Keyword::SET, Keyword::DEFAULT]) {
            Ok(ReferentialAction::SetDefault)
        } else if self.parse_keywords(&[Keyword::NO, Keyword::ACTION]) {
            Ok(ReferentialAction::NoAction)
        } else {
            let found = self.peek_token().cloned();
            self.expected(
                "CASCADE, RESTRICT, SET NULL, SET DEFAULT, or NO ACTION",
                found,
            )
        }
    }

    /// Parses a `ALTER TABLE` statement.
    pub fn parse_alter_table_stmt(&mut self) -> Result<AlterTableStmt, ParserError> {
        self.expect_keywords(&[Keyword::ALTER, Keyword::TABLE])?;
        let if_exists = self.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
        let name = self.parse_object_name()?;
        let action = self.parse_alter_table_action()?;
        Ok(AlterTableStmt {
            if_exists,
            name,
            action,
        })
    }

    /// Parses a `ALTER TABLE` action.
    fn parse_alter_table_action(&mut self) -> Result<AlterTableAction, ParserError> {
        todo!()
    }

    /// Parses a `CREATE VIEW` statement.
    pub fn parse_create_view_stmt(&mut self) -> Result<CreateViewStmt, ParserError> {
        self.expect_keyword(Keyword::CREATE)?;
        let or_replace = self.parse_keywords(&[Keyword::OR, Keyword::REPLACE]);
        let recursive = self.parse_keyword(Keyword::RECURSIVE);
        self.expect_keyword(Keyword::VIEW)?;
        let if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_object_name()?;
        // TODO: columns
        let columns = Vec::new();
        self.expect_keyword(Keyword::AS)?;
        let query = Box::new(self.parse_query_expr(true)?);
        let check_option = self.parse_view_check_option()?;

        Ok(CreateViewStmt {
            or_replace,
            recursive,
            if_not_exists,
            name,
            columns,
            query,
            check_option,
        })
    }

    /// Parses `WITH [ CASCADED | LOCAL  ] CHECK OPTION`
    fn parse_view_check_option(&mut self) -> Result<Option<ViewCheckOption>, ParserError> {
        if self.parse_keyword(Keyword::WITH) {
            let check_option =
                match self.parse_one_of_keywords(&[Keyword::CASCADED, Keyword::LOCAL]) {
                    Some(Keyword::CASCADED) => Some(ViewCheckOption::Cascaded),
                    Some(Keyword::LOCAL) => Some(ViewCheckOption::Local),
                    Some(_) => {
                        let found = self.peek_token().cloned();
                        return self.expected("CASCADED or LOCAL", found);
                    }
                    None => None,
                };
            self.expect_keywords(&[Keyword::CHECK, Keyword::OPTION])?;
            Ok(check_option)
        } else {
            Ok(None)
        }
    }

    /// Parses a `CREATE DOMAIN` statement.
    pub fn parse_create_domain_stmt(&mut self) -> Result<CreateDomainStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::DOMAIN])?;
        let name = self.parse_object_name()?;
        self.parse_keyword(Keyword::AS);
        let data_type = self.parse_data_type()?;
        let constraints = self.parse_comma_separated(Self::parse_domain_constraint_def)?;
        Ok(CreateDomainStmt {
            name,
            data_type,
            constraints,
        })
    }

    /// Parses a domain constraint definition.
    fn parse_domain_constraint_def(&mut self) -> Result<DomainConstraintDef, ParserError> {
        let name = if self.parse_keyword(Keyword::CONSTRAINT) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        let constraint = self.parse_domain_constraint()?;
        Ok(DomainConstraintDef { name, constraint })
    }

    /// Parses a domain constraint.
    fn parse_domain_constraint(&mut self) -> Result<DomainConstraint, ParserError> {
        if self.parse_keyword(Keyword::NULL) {
            Ok(DomainConstraint::Null)
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::NULL]) {
            Ok(DomainConstraint::NotNull)
        } else if self.parse_keyword(Keyword::CHECK) {
            let expr = Box::new(self.parse_expr()?);
            Ok(DomainConstraint::Check(expr))
        } else if self.parse_keyword(Keyword::DEFAULT) {
            let default = self.parse_literal()?;
            Ok(DomainConstraint::Default(default))
        } else if self.parse_keyword(Keyword::COLLATE) {
            let collation = self.parse_object_name()?;
            Ok(DomainConstraint::Collation(collation))
        } else {
            let found = self.peek_token().cloned();
            self.expected("NULL, NOT NULL, CHECK, DEFAULT or COLLATE", found)
        }
    }

    /// Parses a `ALTER DOMAIN` statement.
    pub fn parse_alter_domain_stmt(&mut self) -> Result<AlterDomainStmt, ParserError> {
        self.expect_keywords(&[Keyword::ALTER, Keyword::DOMAIN])?;
        let name = self.parse_object_name()?;
        let action = self.parse_alter_domain_action()?;
        Ok(AlterDomainStmt { name, action })
    }

    /// Parses an `ALTER DOMAIN` action.
    fn parse_alter_domain_action(&mut self) -> Result<AlterDomainAction, ParserError> {
        if self.parse_keywords(&[Keyword::SET, Keyword::DEFAULT]) {
            Ok(AlterDomainAction::SetDefault(self.parse_literal()?))
        } else if self.parse_keywords(&[Keyword::DROP, Keyword::DEFAULT]) {
            Ok(AlterDomainAction::DropDefault)
        } else if self.parse_keyword(Keyword::ADD) {
            Ok(AlterDomainAction::AddConstraint(
                self.parse_domain_constraint_def()?,
            ))
        } else if self.parse_keywords(&[Keyword::DROP, Keyword::CONSTRAINT]) {
            let name = self.parse_identifier()?;
            Ok(AlterDomainAction::DropConstraint(name))
        } else {
            let found = self.peek_token().cloned();
            self.expected(
                "SET DEFAULT, DROP DEFAULT, ADD CONSTRAINT, DROP CONSTRAINT",
                found,
            )
        }
    }

    /// Parses a `CREATE TYPE` statement.
    pub fn parse_create_type_stmt(&mut self) -> Result<CreateTypeStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::TYPE])?;
        let name = self.parse_object_name()?;
        let definition = self.parse_type_definition()?;
        Ok(CreateTypeStmt { name, definition })
    }

    fn parse_type_definition(&mut self) -> Result<Option<TypeDef>, ParserError> {
        todo!()
    }

    /// Parses a `ALTER TYPE` statement.
    pub fn parse_alter_type_stmt(&mut self) -> Result<AlterTypeStmt, ParserError> {
        todo!()
    }

    /// Parses a `CREATE DATABASE` statement.
    pub fn parse_create_database_stmt(&mut self) -> Result<CreateDatabaseStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::DATABASE])?;
        let if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);
        let name = self.parse_object_name()?;
        // TODO:
        let options = Vec::new();
        Ok(CreateDatabaseStmt {
            if_not_exists,
            name,
            options,
        })
    }

    /// Parses a `CREATE INDEX` statement.
    pub fn parse_create_index_stmt(&mut self) -> Result<CreateIndexStmt, ParserError> {
        todo!()
    }

    /// Parses a `DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | DATABASE | INDEX }` statement.
    pub fn parse_drop_stmt(&mut self) -> Result<DropStmt, ParserError> {
        self.expect_keyword(Keyword::DROP)?;
        let ty = self.parse_drop_type()?;
        // Many dialects support the non standard `IF EXISTS` clause and allow
        // specifying multiple objects to delete in a single statement
        let if_exists = self.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
        let names = self.parse_comma_separated(Self::parse_object_name)?;
        let behavior = self.parse_drop_behavior()?;
        Ok(DropStmt {
            ty,
            if_exists,
            names,
            behavior,
        })
    }

    /// Parses drop type.
    pub fn parse_drop_type(&mut self) -> Result<ObjectType, ParserError> {
        match self.parse_one_of_keywords(&[
            Keyword::SCHEMA,
            Keyword::TABLE,
            Keyword::VIEW,
            Keyword::DOMAIN,
            Keyword::TYPE,
            Keyword::DATABASE,
            Keyword::INDEX,
        ]) {
            Some(keyword) => Ok(match keyword {
                Keyword::SCHEMA => ObjectType::Schema,
                Keyword::TABLE => ObjectType::Table,
                Keyword::VIEW => ObjectType::View,
                Keyword::DOMAIN => ObjectType::Domain,
                Keyword::TYPE => ObjectType::Type,
                Keyword::DATABASE => ObjectType::Database,
                Keyword::INDEX => ObjectType::Index,
                _ => unreachable!(),
            }),
            None => {
                let found = self.peek_token().cloned();
                self.expected(
                    "SCHEMA, TABLE, VIEW, DOMAIN, TYPE, DATABASE or INDEX after DROP",
                    found,
                )
            }
        }
    }

    /// Parses drop behavior `CASCADE | RESTRICT`.
    fn parse_drop_behavior(&mut self) -> Result<Option<DropBehavior>, ParserError> {
        match self.parse_one_of_keywords(&[Keyword::CASCADE, Keyword::RESTRICT]) {
            Some(Keyword::CASCADE) => Ok(Some(DropBehavior::Cascade)),
            Some(Keyword::RESTRICT) => Ok(Some(DropBehavior::Restrict)),
            Some(_) => {
                let found = self.peek_token().cloned();
                self.expected("CASCADE or RESTRICT", found)
            }
            None => Ok(None),
        }
    }
}
