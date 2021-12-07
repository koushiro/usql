mod table;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec, vec::Vec};

use usql_ast::{expression::*, types::*};
use usql_core::{Dialect, Keyword};
use usql_lexer::Token;

use crate::{error::ParserError, parser::Parser};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parses a query expression.
    ///
    /// ```txt
    /// <query expression> ::= [ <with clause> ] <query expression body>
    ///     [ <order by clause> ]
    ///     [ <result offset clause> ]
    ///     [ <limit clause> | <fetch first clause> ]
    /// ```
    pub fn parse_query_expr(&mut self, skip_with: bool) -> Result<Query, ParserError> {
        let with = if skip_with { None } else { self.parse_with_clause()? };
        let body = self.parse_query_body(0)?;
        let order_by = self.parse_order_by_clause()?;

        let mut offset = None;
        let mut limit = None;
        let mut fetch = None;
        loop {
            let token = self.peek_token().cloned();
            match token {
                Some(token) if token.is_keyword(Keyword::OFFSET) => {
                    offset = if offset.is_none() {
                        self.parse_offset_clause()?
                    } else {
                        return self
                            .expected("LIMIT or FETCH clause", Some("Duplicated OFFSET clause"));
                    };
                }
                Some(token) if token.is_keyword(Keyword::LIMIT) => {
                    limit = if limit.is_none() && fetch.is_none() {
                        self.parse_limit_clause()?
                    } else {
                        return self.expected("OFFSET clause", Some("LIMIT or FETCH clause"));
                    };
                }
                Some(token) if token.is_keyword(Keyword::FETCH) => {
                    fetch = if fetch.is_none() && limit.is_none() {
                        self.parse_fetch_clause()?
                    } else {
                        return self.expected("OFFSET clause", Some("FETCH or LIMIT clause"));
                    };
                }
                _ => break,
            }
        }

        Ok(Query {
            with,
            body,
            order_by,
            offset,
            limit,
            fetch,
        })
    }

    // ========================================================================
    // query expression body
    // ========================================================================

    /// Parses a query expression body.
    ///
    /// ```txt
    /// <query expression body> ::=
    ///     <query term>
    ///     | <query expression body> UNION [ ALL | DISTINCT] <query expression body>
    ///     | <query expression body> INTERSECT [ ALL | DISTINCT] <query expression body>
    ///
    /// <query term> ::= <query primary> | <query term> INTERSECT [ ALL | DISTINCT] <query primary>
    /// <query primary> ::= <simple table> | no-with-clause query expression
    ///
    /// <simple table> ::= <query specification> | <table value constructor> | <explicit table>
    /// <table value constructor> ::= VALUES <row value expression> [ , ... ]
    /// <explicit table> ::= TABLE <table or query name>
    /// ```
    fn parse_query_body(&mut self, precedence: u8) -> Result<QueryBody, ParserError> {
        let mut body = match self.peek_token().cloned() {
            Some(token) if token.is_keyword(Keyword::SELECT) => {
                let select = self.parse_query_spec()?;
                QueryBody::QuerySpec(Box::new(select))
            }
            Some(token) if token == Token::LeftParen => {
                // with clause are not allowed here
                self.next_token();
                let subquery = self.parse_query_expr(true)?;
                self.expect_token(&Token::RightParen)?;
                QueryBody::Subquery(Box::new(subquery))
            }
            Some(token) if token.is_keyword(Keyword::VALUES) => {
                let list = Default::default();
                QueryBody::Values(Values { list })
            }
            Some(token) if token.is_keyword(Keyword::TABLE) => {
                self.next_token(); // consume the keyword `TABLE`
                let name = self.parse_object_name()?;
                QueryBody::Table(name)
            }
            unexpected => {
                return self.expected("SELECT, Subquery, VALUES or TABLE", unexpected);
            }
        };

        loop {
            // The query can be optionally followed by a set operator
            let token = self.peek_token().cloned();
            let op = self.parse_query_body_operator(token);
            let next_precedence = match op {
                // UNION and EXCEPT have the same binding power and evaluate left-to-right
                Some(QueryBodyOperator::Union) | Some(QueryBodyOperator::Except) => 10,
                // INTERSECT has higher precedence than UNION/EXCEPT
                Some(QueryBodyOperator::Intersect) => 20,
                // Unexpected token or EOF => stop parsing the query body
                None => break,
            };
            if precedence >= next_precedence {
                break;
            }
            self.next_token(); // consume the query body operator
            body = QueryBody::Operation {
                left: Box::new(body),
                op: op.unwrap(),
                quantifier: self.parse_set_quantifier(),
                right: Box::new(self.parse_query_body(next_precedence)?),
            };
        }

        Ok(body)
    }

    fn parse_query_body_operator(&mut self, token: Option<Token>) -> Option<QueryBodyOperator> {
        match token {
            Some(token) if token.is_keyword(Keyword::UNION) => Some(QueryBodyOperator::Union),
            Some(token) if token.is_keyword(Keyword::EXCEPT) => Some(QueryBodyOperator::Except),
            Some(token) if token.is_keyword(Keyword::INTERSECT) => {
                Some(QueryBodyOperator::Intersect)
            }
            _ => None,
        }
    }

    /// Parses a query specification.
    ///
    /// ```txt
    /// <query specification> ::= SELECT [ ALL | DISTINCT ] <select list> <table expression>
    ///
    /// <table expression> ::= <from clause>
    ///     [ <where clause> ]
    ///     [ <group by clause> ]
    ///     [ <having clause> ]
    ///     [ <window clause> ]
    /// ```
    pub fn parse_query_spec(&mut self) -> Result<QuerySpec, ParserError> {
        self.expect_keyword(Keyword::SELECT)?;
        let quantifier = self.parse_set_quantifier();
        let projection = self.parse_comma_separated(Self::parse_select_item)?;

        // table expression
        let from = self.parse_from_clause()?;
        let r#where = self.parse_where_clause()?;
        let group_by = self.parse_group_by_clause()?;
        let having = self.parse_having_clause()?;
        let window = self.parse_window_clause()?;

        Ok(QuerySpec {
            quantifier,
            projection,
            from,
            r#where,
            group_by,
            having,
            window,
        })
    }

    /// Parses a set quantifier.
    pub fn parse_set_quantifier(&mut self) -> Option<SetQuantifier> {
        match self.peek_token() {
            Some(token) if token.is_keyword(Keyword::ALL) => {
                self.next_token(); // consume the `ALL` keyword
                Some(SetQuantifier::All)
            }
            Some(token) if token.is_keyword(Keyword::DISTINCT) => {
                self.next_token(); // consume the `DISTINCT` keyword
                Some(SetQuantifier::Distinct)
            }
            _ => None,
        }
    }

    /// Parses one item of select list.
    ///
    /// ```txt
    /// <select list> ::= * | <select sublist>  [ , ... ]
    ///
    /// <select sublist> ::= <qualified asterisk> | <derived column>
    /// <qualified asterisk> ::= <ident> [ . ... ] .*
    /// <derived column> ::= <expression> [ AS <column name> ]
    /// ```
    pub fn parse_select_item(&mut self) -> Result<SelectItem, ParserError> {
        match self.parse_expr()? {
            Expr::Wildcard => Ok(SelectItem::Wildcard),
            Expr::QualifiedWildcard(prefix) => {
                let name = ObjectName(prefix);
                Ok(SelectItem::QualifiedWildcard(name))
            }
            expr => {
                let alias = if self.parse_keyword(Keyword::AS) {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                Ok(SelectItem::DerivedColumn {
                    expr: Box::new(expr),
                    alias,
                })
            }
        }
    }

    // ========================================================================
    // with clause
    // ========================================================================

    /// Parses a `WITH` clause.
    ///
    /// ```txt
    /// <with clause> ::= WITH [ RECURSIVE ] <with list>
    /// <with list> ::= <with list element> [ , ... ]
    /// <with list element> ::= <query name> [ ( <column list> ) ] AS ( <query expression> )
    /// ```
    pub fn parse_with_clause(&mut self) -> Result<Option<With>, ParserError> {
        if self.parse_keyword(Keyword::WITH) {
            let recursive = self.parse_keyword(Keyword::RECURSIVE);
            let ctes = self.parse_comma_separated(Self::parse_cte)?;
            Ok(Some(With { recursive, ctes }))
        } else {
            Ok(None)
        }
    }

    /// Parses a common table expression.
    ///
    /// ```txt
    /// <with list element> ::= <query name> [ ( <column list> ) ] AS ( <query expression> )
    /// ```
    pub fn parse_cte(&mut self) -> Result<Cte, ParserError> {
        // `<name> [ col1 [, ...] ]`
        let name = self.parse_identifier()?;
        let columns = self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
        // `AS ( <no-with-clause query> )`
        self.expect_keyword(Keyword::AS)?;
        self.expect_token(&Token::LeftParen)?;
        let query = Box::new(self.parse_query_expr(true)?);
        self.expect_token(&Token::RightParen)?;
        Ok(Cte { name, columns, query })
    }

    // ========================================================================
    // order by clause
    // ========================================================================

    /// Parses an `ORDER BY` clause.
    ///
    /// ```txt
    /// <order by clause> ::= ORDER BY <sort specification> [ , ... ]
    /// ```
    pub fn parse_order_by_clause(&mut self) -> Result<Option<OrderBy>, ParserError> {
        if self.parse_keywords(&[Keyword::ORDER, Keyword::BY]) {
            let list = self.parse_comma_separated(Self::parse_sort_spec)?;
            Ok(Some(OrderBy { list }))
        } else {
            Ok(None)
        }
    }

    /// Parses a sort specification.
    ///
    /// ```txt
    /// <sort specification> ::= <sort key> [ ASC | DESC ] [ NULLS FIRST | NULLS LAST ]
    /// ```
    pub fn parse_sort_spec(&mut self) -> Result<SortSpec, ParserError> {
        let expr = self.parse_expr()?;

        let asc = if self.parse_keyword(Keyword::ASC) {
            Some(true)
        } else if self.parse_keyword(Keyword::DESC) {
            Some(false)
        } else {
            None
        };

        let nulls_first = if self.parse_keywords(&[Keyword::NULLS, Keyword::FIRST]) {
            Some(true)
        } else if self.parse_keywords(&[Keyword::NULLS, Keyword::LAST]) {
            Some(false)
        } else {
            None
        };

        Ok(SortSpec {
            expr: Box::new(expr),
            asc,
            nulls_first,
        })
    }

    // ========================================================================
    // limit clause (Not ANSI SQL standard, but most dialects support it)
    // ========================================================================

    /// Parses a `LIMIT` clause.
    ///
    /// ```txt
    /// <limit clause> ::= LIMIT <count>
    /// ```
    pub fn parse_limit_clause(&mut self) -> Result<Option<Limit>, ParserError> {
        if self.parse_keyword(Keyword::LIMIT) {
            if self.parse_keyword(Keyword::ALL) {
                // PostgreSQL-specific, `LIMIT ALL`
                Ok(None)
            } else {
                // `LIMIT <count>`
                let count = self.parse_literal()?;
                Ok(Some(Limit { count }))
            }
        } else if self.next_token_if_is(&Token::word::<D::Keyword, _>("LIMIT", None)) {
            // NOTE: most dialects support `LIMIT` clause, but ANSI SQL don't support it.
            let _count = self.parse_literal()?;
            Ok(None)
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // result offset clause
    // ========================================================================

    /// Parses an `OFFSET` clause.
    ///
    /// ```txt
    /// <result offset clause> ::= OFFSET <count> [ ROW | ROWS ]
    /// ```
    pub fn parse_offset_clause(&mut self) -> Result<Option<Offset>, ParserError> {
        if self.parse_keyword(Keyword::OFFSET) {
            let offset = self.parse_literal()?;
            let rows = if self.parse_keyword(Keyword::ROW) {
                OffsetRows::Row
            } else if self.parse_keyword(Keyword::ROWS) {
                OffsetRows::Rows
            } else {
                OffsetRows::None
            };
            Ok(Some(Offset {
                count: offset,
                rows,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // fetch first clause
    // ========================================================================

    /// Parses a `FETCH FIRST` clause.
    ///
    /// ```txt
    /// <fetch first clause> ::= FETCH [ FIRST | NEXT ] <fetch first quantity> { ROW | ROWS } { ONLY | WITH TIES }
    /// <fetched first quantity> ::= <quantity> [ PERCENT ]
    /// ```
    pub fn parse_fetch_clause(&mut self) -> Result<Option<Fetch>, ParserError> {
        if self.parse_keyword(Keyword::FETCH) {
            self.expect_one_of_keywords(&[Keyword::FIRST, Keyword::NEXT])?;

            let (quantity, percent) = if self
                .parse_one_of_keywords(&[Keyword::ROW, Keyword::ROWS])
                .is_some()
            {
                (None, false)
            } else {
                let quantity = self.parse_literal()?;
                let percent = self.parse_keyword(Keyword::PERCENT);
                self.expect_one_of_keywords(&[Keyword::ROW, Keyword::ROWS])?;
                (Some(quantity), percent)
            };

            let with_ties = if self.parse_keyword(Keyword::ONLY) {
                false
            } else if self.parse_keywords(&[Keyword::WITH, Keyword::TIES]) {
                true
            } else {
                let found = self.peek_token().cloned();
                return self.expected("one of ONLY or WITH TIES", found);
            };

            Ok(Some(Fetch {
                with_ties,
                percent,
                quantity,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query() -> Result<(), ParserError> {
        Ok(())
    }

    #[test]
    fn parse_query_body() -> Result<(), ParserError> {
        Ok(())
    }

    #[test]
    fn parse_with() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        let query = Box::new(Query {
            with: None,
            body: QueryBody::QuerySpec(Box::new(QuerySpec {
                quantifier: None,
                projection: vec![
                    SelectItem::DerivedColumn {
                        expr: Box::new(Expr::Identifier(Ident::new("id1"))),
                        alias: None,
                    },
                    SelectItem::DerivedColumn {
                        expr: Box::new(Expr::Identifier(Ident::new("id2"))),
                        alias: None,
                    },
                ],
                from: From {
                    list: vec![TableReference {
                        relation: TableFactor::Table {
                            name: ObjectName::new(vec!["table1"]),
                            alias: None,
                        },
                        joins: vec![],
                    }],
                },
                r#where: None,
                group_by: None,
                having: None,
                window: None,
            })),
            order_by: None,
            limit: None,
            offset: None,
            fetch: None,
        });
        let sql = "x AS (SELECT id1, id2 FROM table1)";
        let cte = Parser::new_with_sql(&dialect, sql)?.parse_cte()?;
        assert_eq!(cte.name, Ident::new("x"));
        assert_eq!(cte.columns, None);
        // assert_eq!(
        //     Parser::new_with_sql(&dialect, sql)?.parse_cte()?,
        //     Cte {
        //         name: Ident::new("x"),
        //         columns: None,
        //         query: query.clone(),
        //     },
        // );
        // let sql = "WITH RECURSIVE x AS (SELECT id1, id2 FROM table1), y (col1, col2) AS (SELECT id1, id2 FROM table1)";
        // assert_eq!(
        //     Parser::new_with_sql(&dialect, sql)?.parse_with_clause()?,
        //     Some(With {
        //         recursive: true,
        //         ctes: vec![
        //             Cte {
        //                 alias: TableAlias {
        //                     name: Ident::new("x"),
        //                     columns: None,
        //                 },
        //                 query: query.clone(),
        //             },
        //             Cte {
        //                 alias: TableAlias {
        //                     name: Ident::new("y"),
        //                     columns: Some(vec![Ident::new("col1"), Ident::new("col2")]),
        //                 },
        //                 query,
        //             },
        //         ]
        //     })
        // );
        Ok(())
    }

    #[test]
    fn parse_order_by() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ORDER BY id1")?.parse_order_by_clause()?,
            Some(OrderBy {
                list: vec![SortSpec {
                    expr: Box::new(Expr::Identifier(Ident::new("id1"))),
                    asc: None,
                    nulls_first: None,
                }]
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ORDER BY id1 DESC NULLS LAST")?
                .parse_order_by_clause()?,
            Some(OrderBy {
                list: vec![SortSpec {
                    expr: Box::new(Expr::Identifier(Ident::new("id1"))),
                    asc: Some(false),
                    nulls_first: Some(false),
                }]
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "ORDER BY id1 DESC NULLS LAST, id2 ASC")?
                .parse_order_by_clause()?,
            Some(OrderBy {
                list: vec![
                    SortSpec {
                        expr: Box::new(Expr::Identifier(Ident::new("id1"))),
                        asc: Some(false),
                        nulls_first: Some(false),
                    },
                    SortSpec {
                        expr: Box::new(Expr::Identifier(Ident::new("id2"))),
                        asc: Some(true),
                        nulls_first: None,
                    }
                ]
            })
        );
        Ok(())
    }

    #[test]
    fn parse_limit() -> Result<(), ParserError> {
        let dialect = usql_core::postgres::PostgresDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "LIMIT ALL")?.parse_limit_clause()?,
            None
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "LIMIT 1")?.parse_limit_clause()?,
            Some(Limit {
                count: Literal::Number("1".into())
            })
        );
        Ok(())
    }

    #[test]
    fn parse_offset() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "OFFSET 1 ROW")?.parse_offset_clause()?,
            Some(Offset {
                count: Literal::Number("1".into()),
                rows: OffsetRows::Row,
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "OFFSET 2 ROWS")?.parse_offset_clause()?,
            Some(Offset {
                count: Literal::Number("2".into()),
                rows: OffsetRows::Rows,
            })
        );
        let dialect = usql_core::mysql::MysqlDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "OFFSET 1")?.parse_offset_clause()?,
            Some(Offset {
                count: Literal::Number("1".into()),
                rows: OffsetRows::None,
            })
        );
        Ok(())
    }

    #[test]
    fn parse_fetch() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "FETCH FIRST 1 ROW ONLY")?.parse_fetch_clause()?,
            Some(Fetch {
                quantity: Some(Literal::Number("1".into())),
                percent: false,
                with_ties: false,
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "FETCH FIRST 2 PERCENT ROWS ONLY")?
                .parse_fetch_clause()?,
            Some(Fetch {
                quantity: Some(Literal::Number("2".into())),
                percent: true,
                with_ties: false,
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "FETCH FIRST 2 ROWS WITH TIES")?.parse_fetch_clause()?,
            Some(Fetch {
                quantity: Some(Literal::Number("2".into())),
                percent: false,
                with_ties: true,
            })
        );
        Ok(())
    }
}
