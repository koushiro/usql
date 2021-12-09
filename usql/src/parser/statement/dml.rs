#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::{
    ast::statement::*, dialect::Dialect, error::ParserError, keywords::Keyword, parser::Parser,
    tokens::Token,
};

impl<'a, D: Dialect> Parser<'a, D> {
    // ========================================================================
    // insert statement
    // ========================================================================

    /// Parses a `INSERT` statement.
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
    pub fn parse_insert_stmt(&mut self) -> Result<InsertStmt, ParserError> {
        self.expect_keywords(&[Keyword::INSERT, Keyword::INTO])?;
        let table = self.parse_object_name()?;
        if self.parse_keywords(&[Keyword::DEFAULT, Keyword::VALUES]) {
            // <from default>
            let source = InsertSource::Default;
            Ok(InsertStmt { table, source })
        } else {
            let columns = self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
            let overriding = self.parse_optional_insert_overriding_clause()?;
            // <from subquery> or <from constructor>
            match self.peek_token().cloned() {
                Some(token) if token.is_keyword(Keyword::SELECT) => {
                    let subquery = Box::new(self.parse_query_expr(true)?);
                    let source = InsertSource::Subquery {
                        columns,
                        overriding,
                        subquery,
                    };
                    Ok(InsertStmt { table, source })
                }
                Some(token) if token.is_keyword(Keyword::VALUES) => {
                    let values = self.parse_table_values()?;
                    let source = InsertSource::Values {
                        columns,
                        overriding,
                        values,
                    };
                    Ok(InsertStmt { table, source })
                }
                _ => {
                    let found = self.peek_token().cloned();
                    self.expected("insert source", found)
                }
            }
        }
    }

    /// Parses a optional insertion overriding clause.
    ///
    /// ```txt
    /// <overriding clause> ::= OVERRIDING { SYSTEM | USER } VALUES
    /// ```
    pub fn parse_optional_insert_overriding_clause(
        &mut self,
    ) -> Result<Option<InsertOverriding>, ParserError> {
        if self.parse_keyword(Keyword::OVERRIDING) {
            if let Some(kw) = self.parse_one_of_keywords(&[Keyword::SYSTEM, Keyword::USER]) {
                let overriding = match kw {
                    Keyword::SYSTEM => InsertOverriding::System,
                    Keyword::USER => InsertOverriding::User,
                    _ => unreachable!(),
                };
                self.expect_keyword(Keyword::VALUES)?;
                Ok(Some(overriding))
            } else {
                let found = self.peek_token().cloned();
                self.expected("SYSTEM or USER after OVERRIDING", found)
            }
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // delete statement
    // ========================================================================

    /// Parses a `DELETE` statement.
    ///
    /// ```txt
    /// <delete statement> ::= DELETE FROM <table name> [ [ AS ] <alias> ] [ WHERE <search condition> ]
    /// ```
    pub fn parse_delete_stmt(&mut self) -> Result<DeleteStmt, ParserError> {
        self.expect_keywords(&[Keyword::DELETE, Keyword::FROM])?;
        let table = self.parse_object_name()?;
        match self.peek_token() {
            Some(token) if token.is_keyword(Keyword::WHERE) => {
                let selection = self.parse_where_clause()?;
                Ok(DeleteStmt {
                    table,
                    alias: None,
                    selection,
                })
            }
            _ => {
                self.parse_keyword(Keyword::AS);
                let alias = self.parse_identifier()?;
                let selection = self.parse_where_clause()?;
                Ok(DeleteStmt {
                    table,
                    alias: Some(alias),
                    selection,
                })
            }
        }
    }

    // ========================================================================
    // update statement
    // ========================================================================

    /// Parses a `UPDATE` statement.
    ///
    /// ```txt
    /// <update statement> ::= UPDATE <table name> [ [ AS] <alias> ]
    ///     SET <set clause> [ { , <set clause> }... ] [ WHERE <search condition> ]
    ///
    /// <set clause> ::= <multiple column assignment> | <set target> = <update source>
    ///
    /// <multiple column assignment> ::= ( <set target> [ { , <set target> }...] ) = <assigned row>
    /// ```
    pub fn parse_update_stmt(&mut self) -> Result<UpdateStmt, ParserError> {
        self.expect_keyword(Keyword::UPDATE)?;
        let table = self.parse_object_name()?;
        if self.parse_keyword(Keyword::SET) {
            let assignments = self.parse_comma_separated(Self::parse_assignment)?;
            let selection = self.parse_where_clause()?;
            Ok(UpdateStmt {
                table,
                alias: None,
                assignments,
                selection,
            })
        } else {
            self.parse_keyword(Keyword::AS);
            let alias = self.parse_identifier()?;
            self.expect_keyword(Keyword::SET)?;
            let assignments = self.parse_comma_separated(Self::parse_assignment)?;
            let selection = self.parse_where_clause()?;
            Ok(UpdateStmt {
                table,
                alias: Some(alias),
                assignments,
                selection,
            })
        }
    }

    /// Parses a set clause `target = expr`, used in an UPDATE statement.
    pub fn parse_assignment(&mut self) -> Result<Assignment, ParserError> {
        let target = self.parse_identifier()?;
        self.expect_token(&Token::Equal)?;
        let value = Box::new(self.parse_expr()?);
        Ok(Assignment { target, value })
    }

    // ========================================================================
    // select statement
    // ========================================================================

    /// Parses a `SELECT` statement.
    ///
    /// See query expression for details.
    pub fn parse_select_stmt(&mut self) -> Result<SelectStmt, ParserError> {
        let query = Box::new(self.parse_query_expr(false)?);
        Ok(SelectStmt(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{expression::*, types::*};

    #[test]
    fn parse_insert_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "INSERT INTO table1 DEFAULT VALUES";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_insert_stmt()?,
            InsertStmt {
                table: ObjectName::new(vec!["table1"]),
                source: InsertSource::Default,
            }
        );
        let sql = "INSERT INTO table1 VALUES ROW(1, 'foo'), ROW(2, 'bar')";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_insert_stmt()?,
            InsertStmt {
                table: ObjectName::new(vec!["table1"]),
                source: InsertSource::Values {
                    columns: None,
                    overriding: None,
                    values: Values {
                        list: vec![
                            vec![
                                Expr::Literal(Literal::Number("1".into())),
                                Expr::Literal(Literal::String("foo".into())),
                            ],
                            vec![
                                Expr::Literal(Literal::Number("2".into())),
                                Expr::Literal(Literal::String("bar".into())),
                            ]
                        ]
                    }
                },
            }
        );
        let sql = "INSERT INTO table1 SELECT * FROM table2 where id < 100";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_insert_stmt()?,
            InsertStmt {
                table: ObjectName::new(vec!["table1"]),
                source: InsertSource::Subquery {
                    columns: None,
                    overriding: None,
                    subquery: Box::new(Query {
                        with: None,
                        body: QueryBody::QuerySpec(Box::new(QuerySpec {
                            quantifier: None,
                            projection: vec![SelectItem::Wildcard],
                            from: Some(From {
                                list: vec![TableReference {
                                    relation: TableFactor::Table {
                                        name: ObjectName(vec![Ident::new("table2")]),
                                        alias: None,
                                    },
                                    joins: vec![]
                                }],
                            }),
                            r#where: Some(Where {
                                expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                                    left: Box::new(Expr::Identifier(Ident::new("id"))),
                                    op: BinaryOperator::Less,
                                    right: Box::new(Expr::Literal(Literal::Number("100".into())))
                                }))
                            }),
                            group_by: None,
                            having: None,
                            window: None
                        })),
                        order_by: None,
                        limit: None,
                        offset: None,
                        fetch: None
                    }),
                },
            }
        );
        Ok(())
    }

    #[test]
    fn parse_delete_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "DELETE FROM table1 AS t1 WHERE col1 = 1";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_delete_stmt()?,
            DeleteStmt {
                table: ObjectName::new(vec!["table1"]),
                alias: Some(Ident::new("t1")),
                selection: Some(Where {
                    expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                        left: Box::new(Expr::Identifier(Ident::new("col1"))),
                        op: BinaryOperator::Equal,
                        right: Box::new(Expr::Literal(Literal::Number("1".into())))
                    }))
                })
            }
        );
        Ok(())
    }

    #[test]
    fn parse_update_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "UPDATE table1 AS t1 SET col1 = 1, col2 = 2 WHERE col3 = 3";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_update_stmt()?,
            UpdateStmt {
                table: ObjectName::new(vec!["table1"]),
                alias: Some(Ident::new("t1")),
                assignments: vec![
                    Assignment {
                        target: Ident::new("col1"),
                        value: Box::new(Expr::Literal(Literal::Number("1".into())))
                    },
                    Assignment {
                        target: Ident::new("col2"),
                        value: Box::new(Expr::Literal(Literal::Number("2".into())))
                    }
                ],
                selection: Some(Where {
                    expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                        left: Box::new(Expr::Identifier(Ident::new("col3"))),
                        op: BinaryOperator::Equal,
                        right: Box::new(Expr::Literal(Literal::Number("3".into())))
                    }))
                })
            }
        );
        Ok(())
    }

    #[test]
    fn parse_select_stmt() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let sql = "SELECT * FROM table1 WHERE col1 = 1 ORDER BY col2 DESC OFFSET 20 ROWS FETCH FIRST 10 ROWS ONLY";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_select_stmt()?,
            SelectStmt(Box::new(Query {
                with: None,
                body: QueryBody::QuerySpec(Box::new(QuerySpec {
                    quantifier: None,
                    projection: vec![SelectItem::Wildcard],
                    from: Some(From {
                        list: vec![TableReference {
                            relation: TableFactor::Table {
                                name: ObjectName::new(vec!["table1"]),
                                alias: None,
                            },
                            joins: vec![]
                        }],
                    }),
                    r#where: Some(Where {
                        expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                            left: Box::new(Expr::Identifier(Ident::new("col1"))),
                            op: BinaryOperator::Equal,
                            right: Box::new(Expr::Literal(Literal::Number("1".into())))
                        }))
                    }),
                    group_by: None,
                    having: None,
                    window: None
                })),
                order_by: Some(OrderBy {
                    list: vec![SortSpec {
                        expr: Box::new(Expr::Identifier(Ident::new("col2"))),
                        asc: Some(false),
                        nulls_first: None
                    }]
                }),
                limit: None,
                offset: Some(Offset {
                    count: Literal::Number("20".into()),
                    rows: OffsetRows::Rows,
                }),
                fetch: Some(Fetch {
                    quantity: Some(Literal::Number("10".into())),
                    percent: false,
                    with_ties: false,
                })
            }))
        );

        let sql = "SELECT * FROM table1 WHERE col1 = 1 ORDER BY col2 DESC OFFSET 20 ROWS LIMIT 100";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_select_stmt()?,
            SelectStmt(Box::new(Query {
                with: None,
                body: QueryBody::QuerySpec(Box::new(QuerySpec {
                    quantifier: None,
                    projection: vec![SelectItem::Wildcard],
                    from: Some(From {
                        list: vec![TableReference {
                            relation: TableFactor::Table {
                                name: ObjectName::new(vec!["table1"]),
                                alias: None,
                            },
                            joins: vec![]
                        }],
                    }),
                    r#where: Some(Where {
                        expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                            left: Box::new(Expr::Identifier(Ident::new("col1"))),
                            op: BinaryOperator::Equal,
                            right: Box::new(Expr::Literal(Literal::Number("1".into())))
                        }))
                    }),
                    group_by: None,
                    having: None,
                    window: None
                })),
                order_by: Some(OrderBy {
                    list: vec![SortSpec {
                        expr: Box::new(Expr::Identifier(Ident::new("col2"))),
                        asc: Some(false),
                        nulls_first: None
                    }]
                }),
                limit: None,
                offset: Some(Offset {
                    count: Literal::Number("20".into()),
                    rows: OffsetRows::Rows,
                }),
                fetch: None,
            }))
        );

        let dialect = crate::postgres::PostgresDialect::default();
        let sql = "SELECT * FROM table1 WHERE col1 = 1 ORDER BY col2 DESC OFFSET 20 ROWS LIMIT 100";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_select_stmt()?,
            SelectStmt(Box::new(Query {
                with: None,
                body: QueryBody::QuerySpec(Box::new(QuerySpec {
                    quantifier: None,
                    projection: vec![SelectItem::Wildcard],
                    from: Some(From {
                        list: vec![TableReference {
                            relation: TableFactor::Table {
                                name: ObjectName::new(vec!["table1"]),
                                alias: None,
                            },
                            joins: vec![]
                        }],
                    }),
                    r#where: Some(Where {
                        expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                            left: Box::new(Expr::Identifier(Ident::new("col1"))),
                            op: BinaryOperator::Equal,
                            right: Box::new(Expr::Literal(Literal::Number("1".into())))
                        }))
                    }),
                    group_by: None,
                    having: None,
                    window: None
                })),
                order_by: Some(OrderBy {
                    list: vec![SortSpec {
                        expr: Box::new(Expr::Identifier(Ident::new("col2"))),
                        asc: Some(false),
                        nulls_first: None
                    }]
                }),
                limit: Some(Limit {
                    count: Literal::Number("100".into()),
                }),
                offset: Some(Offset {
                    count: Literal::Number("20".into()),
                    rows: OffsetRows::Rows,
                }),
                fetch: None,
            }))
        );
        Ok(())
    }
}
