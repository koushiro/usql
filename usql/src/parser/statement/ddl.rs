#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec, vec::Vec};

use crate::{
    ast::statement::*, dialect::Dialect, error::ParserError, keywords::Keyword, parser::Parser,
    tokens::Token,
};

impl<'a, D: Dialect> Parser<'a, D> {
    // ========================================================================
    // table definition
    // ========================================================================

    /// Parses a `CREATE TABLE` statement.
    ///
    /// ```txt
    /// <table definition> ::=
    ///     CREATE [ <table scope> ] TABLE [ IF NOT EXISTS ] <table name> <table content>
    ///         [ ON COMMIT { PRESERVE ROWS | DELETE ROWS | DROP } ]
    ///
    /// <table scope> ::= { GLOBAL | LOCAL } TEMPORARY
    ///
    /// <table content> ::=
    ///     ( <column definition> [, ...] [, ] [ <table constraint definition> [, ...] ] )
    ///     | LIKE <table name> [ <like option> [, ...] ]
    ///     | AS { ( <query expression> ) | <query expression> }
    /// ```
    pub fn parse_create_table_stmt(&mut self) -> Result<CreateTableStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE])?;
        let scope = self.parse_table_scope()?;
        self.expect_keywords(&[Keyword::TABLE])?;
        // Not ANSI SQL
        let if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_object_name()?;
        let content = self.parse_table_content()?;

        let on_commit = if self.parse_keywords(&[Keyword::ON, Keyword::COMMIT]) {
            match self.expect_one_of_keywords(&[
                Keyword::PRESERVE,
                Keyword::DELETE,
                Keyword::DROP,
            ])? {
                Keyword::PRESERVE => {
                    self.expect_keywords(&[Keyword::ROWS])?;
                    Some(OnCommit::PreserveRows)
                }
                Keyword::DELETE => {
                    self.expect_keywords(&[Keyword::ROWS])?;
                    Some(OnCommit::DeleteRows)
                }
                Keyword::DROP => Some(OnCommit::Drop),
                _ => unreachable!(),
            }
        } else {
            None
        };

        Ok(CreateTableStmt {
            scope,
            if_not_exists,
            name,
            content,
            on_commit,
        })
    }

    /// Parses a table scope.
    ///
    /// ```txt
    /// <table scope> ::= { GLOBAL | LOCAL } TEMPORARY
    /// ```
    fn parse_table_scope(&mut self) -> Result<Option<TableScope>, ParserError> {
        match self.parse_one_of_keywords(&[Keyword::GLOBAL, Keyword::LOCAL]) {
            Some(Keyword::GLOBAL) => {
                self.expect_keyword(Keyword::TEMPORARY)?;
                Ok(Some(TableScope::Global))
            }
            Some(Keyword::LOCAL) => {
                self.expect_keyword(Keyword::TEMPORARY)?;
                Ok(Some(TableScope::Local))
            }
            _ => Ok(None),
        }
    }

    /// Parses the content of table definition.
    ///
    /// ```txt
    /// <table content> ::=
    ///     ( <column definition> [, ...] [, ] [ <table constraint definition> [, ...] ] )
    ///     | LIKE <table name> [ <like option> [, ...] ]
    ///     | AS { ( <query expression> ) | <query expression> }
    /// ```
    fn parse_table_content(&mut self) -> Result<TableContent, ParserError> {
        if self.next_token_if_is(&Token::LeftParen) {
            let mut columns = vec![];
            let mut constraints = vec![];
            loop {
                if let Some(constraint) = self.parse_table_constraint()? {
                    constraints.push(constraint);
                } else if let Some(Token::Word(_)) = self.peek_token() {
                    columns.push(self.parse_column_def()?);
                } else {
                    let found = self.peek_token().cloned();
                    return self.expected("column definition or table constraint", found);
                }
                let comma = self.next_token_if_is(&Token::Comma);
                if self.next_token_if_is(&Token::RightParen) {
                    break;
                } else if !comma {
                    let found = self.peek_token().cloned();
                    return self
                        .expected(", or ) after column definition or table constraint", found);
                }
            }
            Ok(TableContent::Definition {
                columns,
                constraints,
            })
        } else if self.parse_keyword(Keyword::LIKE) {
            let table = self.parse_object_name()?;
            let options =
                self.parse_optional_comma_separated(Self::parse_like_option, "like option")?;
            Ok(TableContent::Like(TableLike { table, options }))
        } else if self.parse_keyword(Keyword::AS) {
            self.next_token_if_is(&Token::LeftParen);
            let query = self.parse_query_expr(true)?;
            self.next_token_if_is(&Token::RightParen);
            Ok(TableContent::SubQuery(Box::new(query)))
        } else {
            let found = self.peek_token().cloned();
            self.expected("table content", found)
        }
    }

    /// Parses a column definition.
    ///
    /// ```txt
    /// <column definition> ::= <column name> <data type> [ <column constraint definition> [, ...] ]
    /// <column constraint definition> ::= [ CONSTRAINT <constraint name> ] <column constraint>
    /// ```
    fn parse_column_def(&mut self) -> Result<ColumnDef, ParserError> {
        let name = self.parse_identifier()?;
        let data_type = self.parse_data_type()?;
        let constraints = self.parse_column_constraint_defs()?;
        Ok(ColumnDef {
            name,
            data_type,
            constraints,
        })
    }

    /// Parses a list of column constraint definition.
    ///
    /// ```txt
    /// <column constraint definition> ::= [ CONSTRAINT <constraint name> ] <column constraint>
    /// ```
    fn parse_column_constraint_defs(&mut self) -> Result<Vec<ColumnConstraintDef>, ParserError> {
        self.parse_constraint_defs(Self::parse_column_constraint)
    }

    /// Parses a list of constraint definitions.
    ///
    /// ```txt
    /// <constraint definition> ::= [ CONSTRAINT <constraint name> ] <constraint>
    /// ```
    fn parse_constraint_defs<C, F>(&mut self, f: F) -> Result<Vec<ConstraintDef<C>>, ParserError>
    where
        F: Fn(&mut Self) -> Result<Option<C>, ParserError>,
    {
        let mut defs = vec![];
        loop {
            if self.parse_keyword(Keyword::CONSTRAINT) {
                let name = Some(self.parse_object_name()?);
                if let Some(constraint) = f(self)? {
                    defs.push(ConstraintDef::<C> { name, constraint });
                } else {
                    let found = self.peek_token().cloned();
                    return self.expected("constraint details after CONSTRAINT <name>", found);
                }
            } else if let Some(constraint) = f(self)? {
                defs.push(ConstraintDef::<C> {
                    name: None,
                    constraint,
                });
            } else {
                break;
            }
        }
        Ok(defs)
    }

    /// Parses a constraint definition.
    ///
    /// ```txt
    /// <constraint definition> ::= [ CONSTRAINT <constraint name> ] <constraint>
    /// ```
    fn parse_constraint_def<C, F>(&mut self, f: F) -> Result<ConstraintDef<C>, ParserError>
    where
        F: Fn(&mut Self) -> Result<Option<C>, ParserError>,
    {
        if self.parse_keyword(Keyword::CONSTRAINT) {
            let name = Some(self.parse_object_name()?);
            if let Some(constraint) = f(self)? {
                Ok(ConstraintDef { name, constraint })
            } else {
                let found = self.peek_token().cloned();
                self.expected("constraint details after CONSTRAINT <name>", found)
            }
        } else if let Some(constraint) = f(self)? {
            Ok(ConstraintDef {
                name: None,
                constraint,
            })
        } else {
            let found = self.peek_token().cloned();
            self.expected("constraint", found)
        }
    }

    /// Parses a column constraint.
    ///
    /// ```txt
    ///
    /// <column constraint> ::=
    ///     NULL
    ///     | NOT NULL
    ///     | <unique specification>
    ///     | <check constraint definition>
    ///     | <references specification>
    ///     | <default specification>
    ///     | <collation specification>
    ///
    /// <unique specification> ::= UNIQUE | PRIMARY KEY
    ///
    /// <check constraint definition> ::= CHECK ( <search condition> )
    ///
    /// <references specification> ::= REFERENCES <table name> [ ( <column name> [, ...] ) ]
    ///     [ MATCH { FULL | PARTIAL | SIMPLE } ]
    ///     [ <referential triggered action> ]
    /// <referential triggered action> ::= <update rule> [ <delete rule> ] | <delete rule> [ <update rule> ]
    /// <update rule> ::= ON UPDATE <referential action>
    /// <delete rule> ::= ON DELETE <referential action>
    /// <referential action> ::= CASCADE | SET NULL | SET DEFAULT | RESTRICT | NO ACTION
    /// ```
    fn parse_column_constraint(&mut self) -> Result<Option<ColumnConstraint>, ParserError> {
        if self.parse_keyword(Keyword::NULL) {
            Ok(Some(ColumnConstraint::Null))
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::NULL]) {
            Ok(Some(ColumnConstraint::NotNull))
        } else if self.parse_keyword(Keyword::UNIQUE) {
            Ok(Some(ColumnConstraint::Unique { is_primary: false }))
        } else if self.parse_keywords(&[Keyword::PRIMARY, Keyword::KEY]) {
            Ok(Some(ColumnConstraint::Unique { is_primary: true }))
        } else if self.parse_keyword(Keyword::CHECK) {
            self.expect_token(&Token::LeftParen)?;
            let expr = Box::new(self.parse_expr()?);
            self.expect_token(&Token::RightParen)?;
            Ok(Some(ColumnConstraint::Check(expr)))
        } else if self.parse_keyword(Keyword::REFERENCES) {
            let table = self.parse_object_name()?;
            let columns = self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
            let match_type = self.parse_referential_match_type()?;
            let (on_update, on_delete) = self.parse_referential_triggered_action()?;
            Ok(Some(ColumnConstraint::References {
                table,
                referenced_columns: columns,
                match_type,
                on_update,
                on_delete,
            }))
        } else if self.parse_keyword(Keyword::DEFAULT) {
            let default = self.parse_literal()?;
            Ok(Some(ColumnConstraint::Default(default)))
        } else if self.parse_keyword(Keyword::COLLATE) {
            let collation = self.parse_object_name()?;
            Ok(Some(ColumnConstraint::Collation(collation)))
        } else {
            Ok(None)
        }
    }

    /// Parses a table constraint.
    ///
    /// ```txt
    /// <table constraint> ::=
    ///     <unique constraint definition>
    ///     | <check constraint definition>
    ///     | <referential constraint definition>
    ///
    /// <unique constraint definition> ::= { UNIQUE | PRIMARY KEY } ( <column name> [, ...] )
    ///
    /// <check constraint definition> ::= CHECK ( <search condition> )
    ///
    /// <referential constraint definition> ::= FOREIGN KEY ( <column name> [, ...] ) <references specification>
    /// <references specification> ::= REFERENCES <table name> [ ( <column name> [, ...] ) ]
    ///     [ MATCH { FULL | PARTIAL | SIMPLE } ]
    ///     [ <referential triggered action> ]
    /// <referential triggered action> ::= <update rule> [ <delete rule> ] | <delete rule> [ <update rule> ]
    /// <update rule> ::= ON UPDATE <referential action>
    /// <delete rule> ::= ON DELETE <referential action>
    /// <referential action> ::= CASCADE | SET NULL | SET DEFAULT | RESTRICT | NO ACTION
    /// ```
    fn parse_table_constraint(&mut self) -> Result<Option<TableConstraint>, ParserError> {
        if self.parse_keyword(Keyword::UNIQUE) {
            self.expect_token(&Token::LeftParen)?;
            let columns = self.parse_comma_separated(Self::parse_identifier)?;
            self.expect_token(&Token::RightParen)?;
            Ok(Some(TableConstraint::Unique {
                is_primary: false,
                columns,
            }))
        } else if self.parse_keywords(&[Keyword::PRIMARY, Keyword::KEY]) {
            self.expect_token(&Token::LeftParen)?;
            let columns = self.parse_comma_separated(Self::parse_identifier)?;
            self.expect_token(&Token::RightParen)?;
            Ok(Some(TableConstraint::Unique {
                is_primary: true,
                columns,
            }))
        } else if self.parse_keyword(Keyword::CHECK) {
            self.expect_token(&Token::LeftParen)?;
            let expr = Box::new(self.parse_expr()?);
            self.expect_token(&Token::RightParen)?;
            Ok(Some(TableConstraint::Check(expr)))
        } else if self.parse_keywords(&[Keyword::FOREIGN, Keyword::KEY]) {
            self.expect_token(&Token::LeftParen)?;
            let referencing_columns = self.parse_comma_separated(Self::parse_identifier)?;
            self.expect_token(&Token::RightParen)?;

            let table = self.parse_object_name()?;
            let referenced_columns =
                self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
            let match_type = self.parse_referential_match_type()?;
            let (on_update, on_delete) = self.parse_referential_triggered_action()?;
            Ok(Some(TableConstraint::ForeignKey {
                referencing_columns,
                table,
                referenced_columns,
                match_type,
                on_update,
                on_delete,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_referential_match_type(
        &mut self,
    ) -> Result<Option<ReferentialMatchType>, ParserError> {
        if self.parse_keyword(Keyword::MATCH) {
            match self.parse_one_of_keywords(&[Keyword::FULL, Keyword::PARTIAL, Keyword::SIMPLE]) {
                Some(keyword) => Ok(Some(match keyword {
                    Keyword::FULL => ReferentialMatchType::Full,
                    Keyword::PARTIAL => ReferentialMatchType::Partial,
                    Keyword::SIMPLE => ReferentialMatchType::Simple,
                    _ => unreachable!(),
                })),
                None => {
                    let found = self.peek_token().cloned();
                    self.expected("FULL, PARTIAL or SIMPLE after MATCH", found)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn parse_referential_triggered_action(
        &mut self,
    ) -> Result<(Option<ReferentialAction>, Option<ReferentialAction>), ParserError> {
        if self.parse_keywords(&[Keyword::ON, Keyword::UPDATE]) {
            let on_update = self.parse_referential_action()?;
            if self.parse_keywords(&[Keyword::ON, Keyword::DELETE]) {
                let on_delete = self.parse_referential_action()?;
                Ok((Some(on_update), Some(on_delete)))
            } else {
                Ok((Some(on_update), None))
            }
        } else if self.parse_keywords(&[Keyword::ON, Keyword::DELETE]) {
            let on_delete = self.parse_referential_action()?;
            if self.parse_keywords(&[Keyword::ON, Keyword::UPDATE]) {
                let on_update = self.parse_referential_action()?;
                Ok((Some(on_update), Some(on_delete)))
            } else {
                Ok((None, Some(on_delete)))
            }
        } else {
            Ok((None, None))
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

    /// Parses a like option.
    fn parse_like_option(&mut self) -> Result<Option<LikeOption>, ParserError> {
        if self.parse_keywords(&[Keyword::INCLUDING, Keyword::IDENTITY]) {
            Ok(Some(LikeOption::IncludingIdentity))
        } else if self.parse_keywords(&[Keyword::EXCLUDING, Keyword::IDENTITY]) {
            Ok(Some(LikeOption::ExcludingIdentity))
        } else if self.parse_keywords(&[Keyword::INCLUDING, Keyword::DEFAULTS]) {
            Ok(Some(LikeOption::IncludingDefaults))
        } else if self.parse_keywords(&[Keyword::EXCLUDING, Keyword::DEFAULTS]) {
            Ok(Some(LikeOption::ExcludingDefaults))
        } else if self.parse_keywords(&[Keyword::INCLUDING, Keyword::GENERATED]) {
            Ok(Some(LikeOption::IncludingGenerated))
        } else if self.parse_keywords(&[Keyword::EXCLUDING, Keyword::GENERATED]) {
            Ok(Some(LikeOption::ExcludingGenerated))
        } else {
            Ok(None)
        }
    }

    /// Parses a `ALTER TABLE` statement.
    ///
    /// ```txt
    /// <alter table statement> ::= ALTER TABLE [ IF EXISTS ] <table name> <alter table action>
    /// ```
    pub fn parse_alter_table_stmt(&mut self) -> Result<AlterTableStmt, ParserError> {
        self.expect_keywords(&[Keyword::ALTER, Keyword::TABLE])?;
        // Not ANSI SQL, but supported by PostgreSQL.
        let if_exists = self.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
        let name = self.parse_object_name()?;
        let action = self.parse_alter_table_action()?;
        Ok(AlterTableStmt {
            if_exists,
            name,
            action,
        })
    }

    /// Parses an alter table action.
    ///
    /// ```txt
    /// <alter table action> ::=
    ///     <add column definition>
    ///     | <alter column definition>
    ///     | <drop column definition>
    ///     | <add table constraint definition>
    ///     | <alter table constraint definition>
    ///     | <drop table constraint definition>
    ///     | <add table period definition>
    ///     | <drop table period definition>
    ///     | <add system versioning clause>
    ///     | <drop system versioning clause>
    /// ```
    fn parse_alter_table_action(&mut self) -> Result<AlterTableAction, ParserError> {
        // we support <add column> and <drop column> now yet
        if self.parse_keyword(Keyword::ADD) {
            self.parse_keyword(Keyword::COLUMN);
            let if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);
            let column = self.parse_column_def()?;
            Ok(AlterTableAction::AddColumn {
                if_not_exists,
                column,
            })
        } else if self.parse_keyword(Keyword::DROP) {
            self.parse_keyword(Keyword::COLUMN);
            let if_exists = self.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
            let name = self.parse_identifier()?;
            let drop_behavior = self.parse_drop_behavior()?;
            Ok(AlterTableAction::DropColumn {
                if_exists,
                name,
                drop_behavior,
            })
        } else {
            let found = self.peek_token().cloned();
            self.expected("ADD COLUMN or DROP COLUMN", found)
        }
    }

    // ========================================================================
    // view definition
    // ========================================================================

    /// Parses a `CREATE VIEW` statement.
    ///
    /// ```txt
    /// <view definition> ::= CREATE [ RECURSIVE ] VIEW <table name> <view specification>
    ///     AS <query expression>  [ WITH [ CASCADED | LOCAL ] CHECK OPTION ]
    ///
    /// <view specification> ::= <regular view specification> | <referencable view specification>
    ///
    /// <regular view specification> ::= [ ( <column name> [, ...] ) ]
    ///
    /// // Not support now
    /// <referencable view specification> ::= OF <user-defined type name> [ UNDER <table name> ] [ ( <view element> [, ...] ) ]
    /// <view element> ::= <self-referencing column specification> | <view column option>
    /// ```
    pub fn parse_create_view_stmt(&mut self) -> Result<CreateViewStmt, ParserError> {
        self.expect_keyword(Keyword::CREATE)?;
        let or_replace = self.parse_keywords(&[Keyword::OR, Keyword::REPLACE]);
        let recursive = self.parse_keyword(Keyword::RECURSIVE);
        self.expect_keyword(Keyword::VIEW)?;
        let if_not_exists = self.parse_keywords(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_object_name()?;
        let columns = self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
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

    /// Parses `WITH [ CASCADED | LOCAL ] CHECK OPTION`
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
                    None => Some(ViewCheckOption::None),
                };
            self.expect_keywords(&[Keyword::CHECK, Keyword::OPTION])?;
            Ok(check_option)
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // domain definition
    // ========================================================================

    /// Parses a `CREATE DOMAIN` statement.
    ///
    /// ```txt
    /// <domain definition> ::= CREATE DOMAIN <domain name> [ AS ] <predefined type> [ <domain constraint definition> [, ...] ]
    /// ```
    pub fn parse_create_domain_stmt(&mut self) -> Result<CreateDomainStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::DOMAIN])?;
        let name = self.parse_object_name()?;
        self.parse_keyword(Keyword::AS);
        let data_type = self.parse_data_type()?;
        let constraints = self.parse_domain_constraint_defs()?;
        Ok(CreateDomainStmt {
            name,
            data_type,
            constraints,
        })
    }

    /// Parses a list of domain constraint definitions.
    ///
    /// ```txt
    /// <domain constraint definition> ::= [ CONSTRAINT <constraint name> ] <domain constraint>
    /// ```
    fn parse_domain_constraint_defs(&mut self) -> Result<Vec<DomainConstraintDef>, ParserError> {
        self.parse_constraint_defs(Self::parse_domain_constraint)
    }

    /// Parses a domain constraint.
    ///
    /// ```txt
    /// <domain constraint> ::=
    ///     NULL
    ///     | NOT NULL
    ///     | CHECK ( <search condition> )
    ///     | <default specification>
    ///     | <collation specification>
    /// ```
    fn parse_domain_constraint(&mut self) -> Result<Option<DomainConstraint>, ParserError> {
        if self.parse_keyword(Keyword::NULL) {
            Ok(Some(DomainConstraint::Null))
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::NULL]) {
            Ok(Some(DomainConstraint::NotNull))
        } else if self.parse_keyword(Keyword::CHECK) {
            let expr = Box::new(self.parse_expr()?);
            Ok(Some(DomainConstraint::Check(expr)))
        } else if self.parse_keyword(Keyword::DEFAULT) {
            let default = self.parse_literal()?;
            Ok(Some(DomainConstraint::Default(default)))
        } else if self.parse_keyword(Keyword::COLLATE) {
            let collation = self.parse_object_name()?;
            Ok(Some(DomainConstraint::Collation(collation)))
        } else {
            Ok(None)
        }
    }

    /// Parses a `ALTER DOMAIN` statement.
    ///
    /// ```txt
    /// <alter domain statement> ::= ALTER DOMAIN <domain name> <alter domain action>
    ///```
    pub fn parse_alter_domain_stmt(&mut self) -> Result<AlterDomainStmt, ParserError> {
        self.expect_keywords(&[Keyword::ALTER, Keyword::DOMAIN])?;
        let name = self.parse_object_name()?;
        let action = self.parse_alter_domain_action()?;
        Ok(AlterDomainStmt { name, action })
    }

    /// Parses an `ALTER DOMAIN` action.
    ///
    /// ```txt
    /// <alter domain action> ::=
    ///     <set domain default clause>
    ///     | <drop domain default clause>
    ///     | <add domain constraint definition>
    ///     | <drop domain constraint definition>
    /// ```
    fn parse_alter_domain_action(&mut self) -> Result<AlterDomainAction, ParserError> {
        if self.parse_keywords(&[Keyword::SET, Keyword::DEFAULT]) {
            Ok(AlterDomainAction::SetDefault(self.parse_literal()?))
        } else if self.parse_keywords(&[Keyword::DROP, Keyword::DEFAULT]) {
            Ok(AlterDomainAction::DropDefault)
        } else if self.parse_keyword(Keyword::ADD) {
            let constraint = self.parse_constraint_def(Self::parse_domain_constraint)?;
            Ok(AlterDomainAction::AddConstraint(constraint))
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

    // ========================================================================
    // user-defined type definition
    // ========================================================================

    /// Parses a `CREATE TYPE` statement.
    ///
    /// ```txt
    /// <user-defined type definition> ::= CREATE TYPE <user-defined type body>
    ///
    /// <user-defined type body> ::= <user-defined type name>
    ///     [ UNDER <super type name> ]
    ///     [ AS <representation> ]
    ///     [ <type option> [, ...] ]
    ///     [ <method specification> [, ...] ]
    ///
    /// <representation> ::= <predefined type> | <collection type> | ( <attribution definition> [, ...] )
    /// <type option> ::=
    ///     { INSTANTIABLE | NOT INSTANTIABLE }
    ///     | { FINAL | NOT FINAL }
    ///     | { REF USING <predefined type> | REF FROM ( <attribute name> [, ...] ) | REF IS SYSTEM GENERATED }
    ///     | CAST ( SOURCE AS REF ) WITH <cast to ref identifier>
    ///     | CAST ( REF AS SOURCE ) WITH <cast to type identifier>
    ///     | CAST ( SOURCE AS DISTINCT ) WITH <cast to distinct identifier>
    ///     | CAST ( DISTINCT AS SOURCE ) WITH <cast to source identifier>
    ///
    /// // Not support now
    /// <method specification> ::= <original method specification> | <overriding method specification>
    /// ```
    pub fn parse_create_type_stmt(&mut self) -> Result<CreateTypeStmt, ParserError> {
        self.expect_keywords(&[Keyword::CREATE, Keyword::TYPE])?;
        let name = self.parse_object_name()?;
        let super_name = if self.parse_keyword(Keyword::UNDER) {
            Some(self.parse_object_name()?)
        } else {
            None
        };
        let representation = self.parse_type_representation()?;
        let options =
            self.parse_optional_comma_separated(Self::parse_type_option, "type option")?;
        Ok(CreateTypeStmt {
            name,
            super_name,
            representation,
            options,
        })
    }

    /// Parses a representation of user-defined type.
    ///
    /// ```txt
    /// AS <representation> ::= AS { <predefined type> | <collection type> | ( <attribute definition> [, ...] ) }
    /// ```
    fn parse_type_representation(&mut self) -> Result<Option<TypeRepresentation>, ParserError> {
        if self.parse_keyword(Keyword::AS) {
            if self.next_token_if_is(&Token::LeftParen) {
                let attrs = self.parse_comma_separated(Self::parse_type_attribute_def)?;
                self.expect_token(&Token::RightParen)?;
                Ok(Some(TypeRepresentation::Attributes(attrs)))
            } else {
                let data_type = self.parse_data_type()?;
                Ok(Some(TypeRepresentation::DataType(data_type)))
            }
        } else {
            Ok(None)
        }
    }

    /// Parses a user-defined type option.
    ///
    /// ```txt
    /// <type option> ::=
    ///     { INSTANTIABLE | NOT INSTANTIABLE }
    ///     | { FINAL | NOT FINAL }
    ///     | { REF USING <predefined type> | REF FROM ( <attribute name> [, ...] ) | REF IS SYSTEM GENERATED }
    ///     | CAST ( SOURCE AS REF ) WITH <cast to ref identifier>
    ///     | CAST ( REF AS SOURCE ) WITH <cast to type identifier>
    ///     | CAST ( SOURCE AS DISTINCT ) WITH <cast to distinct identifier>
    ///     | CAST ( DISTINCT AS SOURCE ) WITH <cast to source identifier>
    /// ```
    fn parse_type_option(&mut self) -> Result<Option<TypeOption>, ParserError> {
        if self.parse_keyword(Keyword::INSTANTIABLE) {
            Ok(Some(TypeOption::Instantiable(false)))
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::INSTANTIABLE]) {
            Ok(Some(TypeOption::Instantiable(true)))
        } else if self.parse_keyword(Keyword::FINAL) {
            Ok(Some(TypeOption::Final(false)))
        } else if self.parse_keywords(&[Keyword::NOT, Keyword::FINAL]) {
            Ok(Some(TypeOption::Final(true)))
        } else if self.parse_keyword(Keyword::CAST) {
            self.expect_token(&Token::LeftParen)?;
            if self.parse_keywords(&[Keyword::SOURCE, Keyword::AS, Keyword::REF]) {
                self.expect_token(&Token::RightParen)?;
                self.expect_keyword(Keyword::WITH)?;
                Ok(Some(TypeOption::CastToRef(self.parse_identifier()?)))
            } else if self.parse_keywords(&[Keyword::REF, Keyword::AS, Keyword::SOURCE]) {
                self.expect_token(&Token::RightParen)?;
                self.expect_keyword(Keyword::WITH)?;
                Ok(Some(TypeOption::CastToType(self.parse_identifier()?)))
            } else if self.parse_keywords(&[Keyword::SOURCE, Keyword::AS, Keyword::DISTINCT]) {
                self.expect_token(&Token::RightParen)?;
                self.expect_keyword(Keyword::WITH)?;
                Ok(Some(TypeOption::CastToDistinct(self.parse_identifier()?)))
            } else if self.parse_keywords(&[Keyword::DISTINCT, Keyword::AS, Keyword::SOURCE]) {
                self.expect_token(&Token::RightParen)?;
                self.expect_keyword(Keyword::WITH)?;
                Ok(Some(TypeOption::CastToSource(self.parse_identifier()?)))
            } else {
                let found = self.peek_token().cloned();
                self.expected(
                    "SOURCE AS REF, REF AS SOURCE, SOURCE AS DISTINCT or DISTINCT AS SOURCE",
                    found,
                )
            }
        } else {
            Ok(None)
        }
    }

    /// Parses an attribute definition of user-defined type
    ///
    /// ```txt
    /// <attribute definition> ::= <attribute name> <data type> [ <default clause> ] [ <collate clause> ]
    /// ```
    fn parse_type_attribute_def(&mut self) -> Result<TypeAttributeDef, ParserError> {
        let name = self.parse_identifier()?;
        let data_type = self.parse_data_type()?;
        let default = if self.parse_keyword(Keyword::DEFAULT) {
            Some(self.parse_literal()?)
        } else {
            None
        };
        let collation = if self.parse_keyword(Keyword::COLLATE) {
            Some(self.parse_object_name()?)
        } else {
            None
        };
        Ok(TypeAttributeDef {
            name,
            data_type,
            default,
            collation,
        })
    }

    /// Parses a `ALTER TYPE` statement.
    ///
    /// ```txt
    /// <alter type statement> ::= ALTER TYPE <user-defined type name> <alter type action>
    /// ```
    pub fn parse_alter_type_stmt(&mut self) -> Result<AlterTypeStmt, ParserError> {
        self.expect_keywords(&[Keyword::ALTER, Keyword::TYPE])?;
        let name = self.parse_object_name()?;
        let action = self.parse_alter_type_action()?;
        Ok(AlterTypeStmt { name, action })
    }

    /// Parses an alter type action.
    ///
    /// ```txt
    /// <alter type action> ::=
    ///     <add attribute definition>
    ///     | <drop attribute definition>
    ///     | <add original method specification>
    ///     | <add overriding method specification>
    ///     | <drop method specification>
    /// ```
    fn parse_alter_type_action(&mut self) -> Result<AlterTypeAction, ParserError> {
        if self.parse_keywords(&[Keyword::ADD, Keyword::ATTRIBUTE]) {
            let attr = self.parse_type_attribute_def()?;
            Ok(AlterTypeAction::AddAttribute(attr))
        } else if self.parse_keywords(&[Keyword::DROP, Keyword::ATTRIBUTE]) {
            let name = self.parse_identifier()?;
            Ok(AlterTypeAction::DropAttribute(name))
        } else {
            let found = self.peek_token().cloned();
            self.expected("ADD ATTRIBUTE or DROP ATTRIBUTE", found)
        }
    }

    // ========================================================================
    // drop statement
    // ========================================================================

    /// Parses a `DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | DATABASE | INDEX }` statement.
    ///
    /// ```txt
    /// <drop schema statement> ::= DROP SCHEMA <schema name> [ IF EXISTS ] <drop behavior>
    /// <drop table statement> ::= DROP TABLE <table name> [ IF EXISTS ] <drop behavior>
    /// <drop view statement> ::= DROP VIEW <table name> [ IF EXISTS ] <drop behavior>
    /// <drop domain statement> ::= DROP DOMAIN <domain name> [ IF EXISTS ] <drop behavior>
    /// <drop data type statement> ::= DROP TYPE <type name> [ IF EXISTS ] <drop behavior>
    ///
    /// // Not ANSI SQL
    /// <drop database statement> ::= DROP DATABASE <database name> [ IF EXISTS ] <drop behavior>
    /// <drop index statement> ::= DROP INDEX <index name> [ IF EXISTS ] <drop behavior>
    /// ```
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

    /// Parses a drop behavior.
    ///
    /// ```txt
    /// <drop behavior> ::= CASCADE | RESTRICT
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{expression::*, types::*};

    #[test]
    fn parse_create_table_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "CREATE TABLE foo (bar INT, baz VARCHAR(10), PRIMARY KEY (bar))";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_create_table_stmt()?,
            CreateTableStmt {
                scope: None,
                if_not_exists: false,
                name: ObjectName::new(vec!["foo"]),
                content: TableContent::Definition {
                    columns: vec![
                        ColumnDef {
                            name: Ident::new("bar"),
                            data_type: DataType::Int(None),
                            constraints: vec![],
                        },
                        ColumnDef {
                            name: Ident::new("baz"),
                            data_type: DataType::Varchar(10),
                            constraints: vec![],
                        },
                    ],
                    constraints: vec![TableConstraint::Unique {
                        is_primary: true,
                        columns: vec![Ident::new("bar")],
                    }],
                },
                on_commit: None
            }
        );
        let sql = "CREATE TABLE foo (bar INT PRIMARY KEY, baz VARCHAR(10))";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_create_table_stmt()?,
            CreateTableStmt {
                scope: None,
                if_not_exists: false,
                name: ObjectName::new(vec!["foo"]),
                content: TableContent::Definition {
                    columns: vec![
                        ColumnDef {
                            name: Ident::new("bar"),
                            data_type: DataType::Int(None),
                            constraints: vec![ColumnConstraintDef {
                                name: None,
                                constraint: ColumnConstraint::Unique { is_primary: true }
                            }],
                        },
                        ColumnDef {
                            name: Ident::new("baz"),
                            data_type: DataType::Varchar(10),
                            constraints: vec![],
                        },
                    ],
                    constraints: vec![],
                },
                on_commit: None
            }
        );
        let sql = "CREATE TABLE foo LIKE bar INCLUDING IDENTITY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_create_table_stmt()?,
            CreateTableStmt {
                scope: None,
                if_not_exists: false,
                name: ObjectName::new(vec!["foo"]),
                content: TableContent::Like(TableLike {
                    table: ObjectName::new(vec!["bar"]),
                    options: Some(vec![LikeOption::IncludingIdentity])
                }),
                on_commit: None
            }
        );
        Ok(())
    }

    #[test]
    fn parse_alter_table_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "ALTER TABLE foo ADD COLUMN bar INT";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_alter_table_stmt()?,
            AlterTableStmt {
                if_exists: false,
                name: ObjectName::new(vec!["foo"]),
                action: AlterTableAction::AddColumn {
                    if_not_exists: false,
                    column: ColumnDef {
                        name: Ident::new("bar"),
                        data_type: DataType::Int(None),
                        constraints: vec![],
                    },
                },
            }
        );
        let sql = "ALTER TABLE foo DROP COLUMN bar";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_alter_table_stmt()?,
            AlterTableStmt {
                if_exists: false,
                name: ObjectName::new(vec!["foo"]),
                action: AlterTableAction::DropColumn {
                    if_exists: false,
                    name: Ident::new("bar"),
                    drop_behavior: None,
                },
            }
        );
        Ok(())
    }

    #[test]
    fn parse_create_view_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "CREATE VIEW foo AS SELECT * FROM bar")?
                .parse_create_view_stmt()?,
            CreateViewStmt {
                or_replace: false,
                recursive: false,
                if_not_exists: false,
                name: ObjectName::new(vec!["foo"]),
                columns: None,
                query: Box::new(Query {
                    with: None,
                    body: QueryBody::QuerySpec(Box::new(QuerySpec {
                        quantifier: None,
                        projection: vec![SelectItem::Wildcard],
                        from: Some(From {
                            list: vec![TableReference {
                                relation: TableFactor::Table {
                                    name: ObjectName::new(vec!["bar"]),
                                    alias: None,
                                },
                                joins: vec![]
                            }]
                        }),
                        r#where: None,
                        group_by: None,
                        having: None,
                        window: None
                    })),
                    order_by: None,
                    offset: None,
                    limit: None,
                    fetch: None,
                }),
                check_option: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(
                &dialect,
                "CREATE VIEW foo (id1, id2) AS SELECT * FROM bar WITH CASCADED CHECK OPTION"
            )?
            .parse_create_view_stmt()?,
            CreateViewStmt {
                or_replace: false,
                recursive: false,
                if_not_exists: false,
                name: ObjectName::new(vec!["foo"]),
                columns: Some(vec![Ident::new("id1"), Ident::new("id2")]),
                query: Box::new(Query {
                    with: None,
                    body: QueryBody::QuerySpec(Box::new(QuerySpec {
                        quantifier: None,
                        projection: vec![SelectItem::Wildcard],
                        from: Some(From {
                            list: vec![TableReference {
                                relation: TableFactor::Table {
                                    name: ObjectName::new(vec!["bar"]),
                                    alias: None,
                                },
                                joins: vec![]
                            }]
                        }),
                        r#where: None,
                        group_by: None,
                        having: None,
                        window: None
                    })),
                    order_by: None,
                    offset: None,
                    limit: None,
                    fetch: None,
                }),
                check_option: Some(ViewCheckOption::Cascaded),
            }
        );
        Ok(())
    }

    #[test]
    fn parse_create_domain_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "CREATE DOMAIN foo AS INT")?
                .parse_create_domain_stmt()?,
            CreateDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                data_type: DataType::Int(None),
                constraints: vec![]
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "CREATE DOMAIN foo AS INT NOT NULL DEFAULT 0")?
                .parse_create_domain_stmt()?,
            CreateDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                data_type: DataType::Int(None),
                constraints: vec![
                    DomainConstraintDef {
                        name: None,
                        constraint: DomainConstraint::NotNull
                    },
                    DomainConstraintDef {
                        name: None,
                        constraint: DomainConstraint::Default(Literal::Number("0".into()))
                    }
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_alter_domain_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER DOMAIN foo SET DEFAULT 0")?
                .parse_alter_domain_stmt()?,
            AlterDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterDomainAction::SetDefault(Literal::Number("0".into()))
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER DOMAIN foo DROP DEFAULT")?
                .parse_alter_domain_stmt()?,
            AlterDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterDomainAction::DropDefault
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER DOMAIN foo ADD CONSTRAINT bar NOT NULL")?
                .parse_alter_domain_stmt()?,
            AlterDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterDomainAction::AddConstraint(DomainConstraintDef {
                    name: Some(ObjectName::new(vec!["bar"])),
                    constraint: DomainConstraint::NotNull
                })
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER DOMAIN foo DROP CONSTRAINT bar")?
                .parse_alter_domain_stmt()?,
            AlterDomainStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterDomainAction::DropConstraint(Ident::new("bar"))
            }
        );
        Ok(())
    }

    #[test]
    fn parse_create_type_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "CREATE TYPE foo AS INT")?.parse_create_type_stmt()?,
            CreateTypeStmt {
                name: ObjectName::new(vec!["foo"]),
                super_name: None,
                representation: Some(TypeRepresentation::DataType(DataType::Int(None))),
                options: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "CREATE TYPE foo AS (bar INT default 0, baz INT)")?
                .parse_create_type_stmt()?,
            CreateTypeStmt {
                name: ObjectName::new(vec!["foo"]),
                super_name: None,
                representation: Some(TypeRepresentation::Attributes(vec![
                    TypeAttributeDef {
                        name: Ident::new("bar"),
                        data_type: DataType::Int(None),
                        default: Some(Literal::Number("0".into())),
                        collation: None,
                    },
                    TypeAttributeDef {
                        name: Ident::new("baz"),
                        data_type: DataType::Int(None),
                        default: None,
                        collation: None,
                    }
                ])),
                options: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(
                &dialect,
                "CREATE TYPE foo AS INT CAST ( SOURCE AS REF ) WITH bar, FINAL"
            )?
            .parse_create_type_stmt()?,
            CreateTypeStmt {
                name: ObjectName::new(vec!["foo"]),
                super_name: None,
                representation: Some(TypeRepresentation::DataType(DataType::Int(None))),
                options: Some(vec![
                    TypeOption::CastToRef(Ident::new("bar")),
                    TypeOption::Final(false),
                ])
            }
        );
        Ok(())
    }

    #[test]
    fn parse_alter_type_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER TYPE foo ADD ATTRIBUTE bar INT DEFAULT 0")?
                .parse_alter_type_stmt()?,
            AlterTypeStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterTypeAction::AddAttribute(TypeAttributeDef {
                    name: Ident::new("bar"),
                    data_type: DataType::Int(None),
                    default: Some(Literal::Number("0".into())),
                    collation: None,
                })
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ALTER TYPE foo DROP ATTRIBUTE bar")?
                .parse_alter_type_stmt()?,
            AlterTypeStmt {
                name: ObjectName::new(vec!["foo"]),
                action: AlterTypeAction::DropAttribute(Ident::new("bar"))
            }
        );
        Ok(())
    }

    #[test]
    fn parse_drop_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "DROP TABLE foo CASCADE")?.parse_drop_stmt()?,
            DropStmt {
                ty: ObjectType::Table,
                if_exists: false,
                names: vec![ObjectName::new(vec!["foo"])],
                behavior: Some(DropBehavior::Cascade)
            }
        );
        let dialect = crate::postgres::PostgresDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "DROP TABLE IF EXISTS foo")?.parse_drop_stmt()?,
            DropStmt {
                ty: ObjectType::Table,
                if_exists: true,
                names: vec![ObjectName::new(vec!["foo"])],
                behavior: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DROP TABLE IF EXISTS foo CASCADE")?
                .parse_drop_stmt()?,
            DropStmt {
                ty: ObjectType::Table,
                if_exists: true,
                names: vec![ObjectName::new(vec!["foo"])],
                behavior: Some(DropBehavior::Cascade)
            }
        );
        Ok(())
    }
}
