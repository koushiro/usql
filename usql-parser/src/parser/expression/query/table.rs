#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec, vec::Vec};

use usql_ast::{expression::*, types::ObjectName};
use usql_core::{Dialect, Keyword};
use usql_lexer::Token;

use crate::{error::ParserError, parser::Parser};

impl<'a, D: Dialect> Parser<'a, D> {
    // ========================================================================
    // from clause
    // ========================================================================

    /// Parses a `FROM` clause.
    ///
    /// ```txt
    /// <from clause> ::= FROM <table reference list>
    /// <table reference list> ::= <table reference> [ { , <table reference> }... ]
    /// ```
    pub fn parse_from_clause(&mut self) -> Result<From, ParserError> {
        self.expect_keyword(Keyword::FROM)?;
        let list = self.parse_comma_separated(Self::parse_table_reference)?;
        Ok(From { list })
    }

    /// Parses a table reference.
    ///
    /// ```txt
    /// <table reference> ::= <table factor> | <joined table>
    ///
    /// <table factor> ::= <table or query name> | <derived table> | <parenthesized joined table>
    ///
    /// <joined table> ::= <cross join> | <qualified join> | <natural join>
    /// <cross join> ::= <table reference> CROSS JOIN <table factor>
    /// <natural join> ::= <table reference> NATURAL [ <join type>  ] JOIN <table factor>
    /// <qualified join> ::= <table reference> [ <join type>  ] JOIN <table reference> <join specification>
    ///
    /// <join type> ::= INNER | { LEFT | RIGHT | FULL  [ OUTER ] }
    /// <join specification> ::= ON <search condition> | USING ( <column name list> )
    /// ```
    pub fn parse_table_reference(&mut self) -> Result<TableReference, ParserError> {
        let relation = self.parse_table_factor()?;

        let mut joins = vec![];
        loop {
            if let Some(join) = self.parse_joined_table()? {
                joins.push(join);
            } else {
                break;
            }
        }
        Ok(TableReference { relation, joins })
    }

    /// Parses a table factor.
    ///
    /// ```txt
    /// <table factor> ::= <table or query name> | [ LATERAL ] <derived table> | <parenthesized joined table>
    /// ```
    pub fn parse_table_factor(&mut self) -> Result<TableFactor, ParserError> {
        // [ LATERAL ] <derived table>
        if self.parse_keyword(Keyword::NATURAL) {
            self.parse_derived_table_factor(true)
        } else if self.peek_token() == Some(&Token::LeftParen) {
            // A left paren introduces either a derived table (i.e., a subquery) or a nested join.
            self.parse_derived_table_factor(false)
            // TODO: support nested join
        } else {
            // <table or query name>
            let name = self.parse_object_name()?;
            let alias = if self.parse_keyword(Keyword::AS) {
                self.parse_optional_table_alias()?
            } else {
                None
            };
            Ok(TableFactor::Table { name, alias })
        }
    }

    fn parse_derived_table_factor(&mut self, lateral: bool) -> Result<TableFactor, ParserError> {
        // subquery ::= ( <query expression> )
        self.expect_token(&Token::LeftParen)?;
        let subquery = Box::new(self.parse_query_expr(true)?);
        self.expect_token(&Token::RightParen)?;
        let alias = self.parse_optional_table_alias()?;
        Ok(TableFactor::Derived {
            lateral,
            subquery,
            alias,
        })
    }

    /// Parses an optional table alias.
    ///
    /// ```txt
    /// <table alias> ::= [ AS ] <alias name> ( <columns> )
    /// ```
    pub fn parse_optional_table_alias(&mut self) -> Result<Option<TableAlias>, ParserError> {
        if self.parse_keyword(Keyword::AS) {
            let name = self.parse_identifier()?;
            let columns = self.parse_parenthesized_comma_separated(Self::parse_identifier, true)?;
            Ok(Some(TableAlias { name, columns }))
        } else {
            Ok(None)
        }
    }

    /// Parses a joined table.
    ///
    /// ```txt
    /// <joined table> ::= <cross join> | <qualified join> | <natural join>
    ///
    /// <cross join> ::= <table reference> CROSS JOIN <table factor>
    /// <natural join> ::= <table reference> NATURAL [ <join type>  ] JOIN <table factor>
    /// <qualified join> ::= <table reference> [ <join type>  ] JOIN <table reference> <join specification>
    /// ```
    pub fn parse_joined_table(&mut self) -> Result<Option<Join>, ParserError> {
        if self.parse_keyword(Keyword::CROSS) {
            // CROSS JOIN
            self.expect_keyword(Keyword::JOIN)?;
            let relation = self.parse_table_factor()?;
            let join = JoinOperator::CrossJoin;
            Ok(Some(Join { join, relation }))
        } else {
            let natural = self.parse_keyword(Keyword::NATURAL);
            // `NATURAL [ <join type>  ] JOIN` or `[<join type>  ] JOIN`
            match self.parse_one_of_keywords(&[
                Keyword::JOIN,
                Keyword::INNER,
                Keyword::LEFT,
                Keyword::RIGHT,
                Keyword::FULL,
            ]) {
                Some(Keyword::JOIN) | Some(Keyword::INNER) => {
                    let relation = self.parse_table_factor()?;
                    let join = if natural {
                        JoinOperator::NaturalInnerJoin
                    } else {
                        JoinOperator::InnerJoin(self.parse_join_spec()?)
                    };
                    Ok(Some(Join { join, relation }))
                }
                Some(Keyword::LEFT) => {
                    self.parse_keyword(Keyword::OUTER);
                    let relation = self.parse_table_factor()?;
                    let join = if natural {
                        JoinOperator::NaturalLeftOuterJoin
                    } else {
                        JoinOperator::LeftOuterJoin(self.parse_join_spec()?)
                    };
                    Ok(Some(Join { join, relation }))
                }
                Some(Keyword::RIGHT) => {
                    self.parse_keyword(Keyword::OUTER);
                    let relation = self.parse_table_factor()?;
                    let join = if natural {
                        JoinOperator::NaturalRightOuterJoin
                    } else {
                        JoinOperator::RightOuterJoin(self.parse_join_spec()?)
                    };
                    Ok(Some(Join { join, relation }))
                }
                Some(Keyword::FULL) => {
                    self.parse_keyword(Keyword::OUTER);
                    let relation = self.parse_table_factor()?;
                    let join = if natural {
                        JoinOperator::NaturalFullOuterJoin
                    } else {
                        JoinOperator::FullOuterJoin(self.parse_join_spec()?)
                    };
                    Ok(Some(Join { join, relation }))
                }
                _ if natural => {
                    let found = self.peek_token().cloned();
                    self.expected("join type after NATURAL", found)
                }
                _ => Ok(None),
            }
        }
    }

    /// Parses a join specification.
    ///
    /// ```txt
    /// <join specification> ::= <join condition> | <named columns join>
    /// <join condition> ::= ON <search condition>
    /// <named columns join> ::= USING ( <join column list> )  [ AS <join correlation name>  ]
    /// ```
    pub fn parse_join_spec(&mut self) -> Result<JoinSpec, ParserError> {
        if self.parse_keyword(Keyword::ON) {
            let constraint = Box::new(self.parse_expr()?);
            Ok(JoinSpec::On(constraint))
        } else if self.parse_keyword(Keyword::USING) {
            self.expect_token(&Token::LeftParen)?;
            let columns = self.parse_comma_separated(Self::parse_identifier)?;
            self.expect_token(&Token::RightParen)?;
            let alias = if self.parse_keyword(Keyword::AS) {
                Some(self.parse_identifier()?)
            } else {
                None
            };
            Ok(JoinSpec::Using { columns, alias })
        } else {
            let found = self.peek_token().cloned();
            self.expected("ON or USING after join type", found)
        }
    }

    // ========================================================================
    // where clause
    // ========================================================================

    /// Parse a `WHERE` clause.
    ///
    /// ```txt
    /// <where clause> ::= WHERE <search condition>
    /// ```
    pub fn parse_where_clause(&mut self) -> Result<Option<Where>, ParserError> {
        if self.parse_keyword(Keyword::WHERE) {
            let expr = Box::new(self.parse_expr()?);
            Ok(Some(Where { expr }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // group by clause
    // ========================================================================

    /// Parses a `GROUP BY` clause.
    ///
    /// ```txt
    /// <group by clause> ::= GROUP BY [ DISTINCT | ALL ] <group element> [ , ... ]
    /// ```
    pub fn parse_group_by_clause(&mut self) -> Result<Option<GroupBy>, ParserError> {
        if self.parse_keywords(&[Keyword::GROUP, Keyword::BY]) {
            let quantifier = self.parse_set_quantifier();
            let list = self.parse_comma_separated(Self::parse_grouping_element)?;
            Ok(Some(GroupBy { quantifier, list }))
        } else {
            Ok(None)
        }
    }

    /// Parses a grouping element.
    ///
    /// ```txt
    /// <grouping element> ::=
    ///   <empty grouping set>
    ///   | <ordinary grouping set>
    ///   | <rollup list>
    ///   | <cube list>
    ///   | <grouping sets specification>
    ///
    /// <empty grouping set> ::= ( )
    /// <ordinary grouping set> ::= column | ( column [, ...] )
    /// <rollup list> ::= ROLLUP ( { column | ( column [, ...] ) } [, ...] )
    /// <cube list> ::= CUBE  ( { column | ( column [, ...] ) } [, ...] )
    /// <grouping sets specification> ::= GROUPING SETS ( grouping_element [, ...] )
    /// ```
    pub fn parse_grouping_element(&mut self) -> Result<GroupingElement, ParserError> {
        if self.parse_keyword(Keyword::ROLLUP) {
            let list = self.parse_comma_separated(Self::parse_grouping_set)?;
            Ok(GroupingElement::Rollup(list))
        } else if self.parse_keyword(Keyword::CUBE) {
            let list = self.parse_comma_separated(Self::parse_grouping_set)?;
            Ok(GroupingElement::Cube(list))
        } else if self.parse_keywords(&[Keyword::GROUPING, Keyword::SETS]) {
            self.expect_token(&Token::LeftParen)?;
            let list = self.parse_comma_separated(Self::parse_grouping_element)?;
            self.expect_token(&Token::RightParen)?;
            Ok(GroupingElement::Sets(list))
        } else if self.peek_next_token() == Some(&Token::LeftParen) {
            if self.peek_next_token() == Some(&Token::RightParen) {
                self.next_token(); // consume `(`
                self.next_token(); // consume `)`
                Ok(GroupingElement::Empty)
            } else {
                self.reset_peek_cursor();
                Ok(GroupingElement::OrdinarySet(self.parse_grouping_set()?))
            }
        } else {
            let column = self.parse_object_name()?;
            Ok(GroupingElement::OrdinarySet(GroupingSet::Column(column)))
        }
    }

    /// Parses an ordinary grouping set.
    pub fn parse_grouping_set(&mut self) -> Result<GroupingSet, ParserError> {
        if self.next_token_if_is(&Token::LeftParen) {
            let columns = self.parse_comma_separated(Self::parse_object_name)?;
            self.expect_token(&Token::RightParen)?;
            Ok(GroupingSet::Columns(columns))
        } else {
            let column = self.parse_object_name()?;
            Ok(GroupingSet::Column(column))
        }
    }

    // ========================================================================
    // having clause
    // ========================================================================

    /// Parses a `HAVING` clause.
    ///
    /// ```txt
    /// <having clause> ::= HAVING <search condition>
    /// ```
    pub fn parse_having_clause(&mut self) -> Result<Option<Having>, ParserError> {
        if self.parse_keyword(Keyword::HAVING) {
            let expr = Box::new(self.parse_expr()?);
            Ok(Some(Having { expr }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // window clause
    // ========================================================================

    /// Parses a `WINDOW` clause.
    ///
    /// ```txt
    /// <window clause> ::= WINDOW <window definition> [ { , <window definition> }... ]
    /// ```
    pub fn parse_window_clause(&mut self) -> Result<Option<Window>, ParserError> {
        if self.parse_keyword(Keyword::WINDOW) {
            let def_list = self.parse_comma_separated(Self::parse_window_def)?;
            if def_list.is_empty() {
                return self.expected("window definition list", Option::<Token>::None);
            }
            Ok(Some(Window { list: def_list }))
        } else {
            Ok(None)
        }
    }

    /// Parses a window definition.
    ///
    /// ```txt
    /// <window definition> ::= <window name> [ AS ] <window specification>
    /// ```
    pub fn parse_window_def(&mut self) -> Result<WindowDef, ParserError> {
        let name = self.parse_identifier()?;
        self.expect_keyword(Keyword::AS)?;
        let spec = self.parse_window_spec()?;
        Ok(WindowDef { name, spec })
    }

    /// Parses a window specification.
    ///
    /// ```txt
    /// <window specification> ::= ( <window specification details> )
    /// <window specification details> ::=
    ///     [ <existing window name> ]
    ///     [ <window partition clause> ]
    ///     [ <window order clause> ]
    ///     [ <window frame clause> ]
    /// <window partition clause> ::= PARTITION BY <window partition column> [ { , <window partition column> }... ]
    /// <window order clause> ::= ORDER BY { <sort_key> [ ASC | DESC ] [ NULLS FIRST | NULLS LAST ] } [, ...]`
    /// ```
    pub fn parse_window_spec(&mut self) -> Result<WindowSpec, ParserError> {
        self.expect_token(&Token::LeftParen)?;
        // NOTE: we don't support the existing window name
        // window partition clause
        let partition_by = self.parse_window_partition_clause()?;
        // window order clause
        let order_by = self.parse_order_by_clause()?;
        // window frame clause
        let window_frame = self.parse_window_frame_clause()?;
        self.expect_token(&Token::RightParen)?;
        Ok(WindowSpec {
            name: None,
            partition_by,
            order_by,
            window_frame,
        })
    }

    /// Parses a window partition clause.
    pub fn parse_window_partition_clause(
        &mut self,
    ) -> Result<Option<Vec<ObjectName>>, ParserError> {
        if self.parse_keywords(&[Keyword::PARTITION, Keyword::BY]) {
            // a list of possibly-qualified column names
            Ok(Some(self.parse_comma_separated(Self::parse_object_name)?))
        } else {
            Ok(None)
        }
    }

    /// Parses a window frame clause.
    ///
    /// ```txt
    /// <window frame clause> ::= <window frame units> <window frame extent> [ <window frame exclusion> ]
    ///
    /// <window frame units> ::= ROWS | RANGE | GROUPS
    ///
    /// <window frame extent> ::=  <window frame start> | <window frame between>
    /// <window frame between> ::= BETWEEN <window frame bound>  AND <window frame bound>
    /// <window frame bound> ::=  <window frame start> | UNBOUNDED FOLLOWING | <unsigned integer> FOLLOWING
    /// <window frame start> ::= CURRENT ROW | UNBOUNDED PRECEDING | <unsigned integer> PRECEDING
    ///
    /// <window frame exclusion> ::= EXCLUDE CURRENT ROW | EXCLUDE GROUP | EXCLUDE TIES | EXCLUDE NO OTHERS
    /// ```
    pub fn parse_window_frame_clause(&mut self) -> Result<Option<WindowFrame>, ParserError> {
        match self.peek_token() {
            Some(token)
                if token
                    .is_one_of_keywords(&[Keyword::ROWS, Keyword::RANGE, Keyword::GROUPS])
                    .is_some() =>
            {
                let units = self.parse_window_frame_units()?;
                let (start_bound, end_bound) = if self.parse_keyword(Keyword::BETWEEN) {
                    let start_bound = self.parse_window_frame_bound()?;
                    self.expect_keyword(Keyword::AND)?;
                    let end_bound = Some(self.parse_window_frame_bound()?);
                    (start_bound, end_bound)
                } else {
                    (self.parse_window_frame_bound()?, None)
                };
                let exclusion = self.parse_window_frame_exclusion()?;
                Ok(Some(WindowFrame {
                    units,
                    start_bound,
                    end_bound,
                    exclusion,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Parses a window frame units.
    ///
    /// ```txt
    /// <window frame units> ::= ROWS | RANGE | GROUPS
    /// ```
    pub fn parse_window_frame_units(&mut self) -> Result<WindowFrameUnits, ParserError> {
        match self.next_token() {
            Some(Token::Word(w)) => match w.keyword {
                Some(Keyword::ROWS) => Ok(WindowFrameUnits::Rows),
                Some(Keyword::RANGE) => Ok(WindowFrameUnits::Range),
                Some(Keyword::GROUPS) => Ok(WindowFrameUnits::Groups),
                _ => self.expected("ROWS, RANGE, GROUPS", Some(Token::Word(w)))?,
            },
            unexpected => self.expected("ROWS, RANGE, GROUPS", unexpected),
        }
    }

    /// Parses `CURRENT ROW` or `{ <unsigned integer> | UNBOUNDED } { PRECEDING | FOLLOWING }`
    ///
    /// ```txt
    /// <window frame bound> ::=  <window frame start> | UNBOUNDED FOLLOWING | <unsigned integer> FOLLOWING
    /// <window frame start> ::= CURRENT ROW | UNBOUNDED PRECEDING | <unsigned integer> PRECEDING
    /// ```
    pub fn parse_window_frame_bound(&mut self) -> Result<WindowFrameBound, ParserError> {
        if self.parse_keywords(&[Keyword::CURRENT, Keyword::ROW]) {
            Ok(WindowFrameBound::CurrentRow)
        } else {
            let rows = if self.parse_keyword(Keyword::UNBOUNDED) {
                None
            } else {
                Some(self.parse_literal_uint()?)
            };
            if self.parse_keyword(Keyword::PRECEDING) {
                Ok(WindowFrameBound::Preceding(rows))
            } else if self.parse_keyword(Keyword::FOLLOWING) {
                Ok(WindowFrameBound::Following(rows))
            } else {
                let found = self.peek_token().cloned();
                self.expected("PRECEDING or FOLLOWING", found)
            }
        }
    }

    /// Parses window frame exclusion.
    ///
    /// ```txt
    /// <window frame exclusion> ::= EXCLUDE CURRENT ROW | EXCLUDE GROUP | EXCLUDE TIES | EXCLUDE NO OTHERS
    /// ```
    pub fn parse_window_frame_exclusion(
        &mut self,
    ) -> Result<Option<WindowFrameExclusion>, ParserError> {
        if self.parse_keyword(Keyword::EXCLUDE) {
            if self.parse_keywords(&[Keyword::CURRENT, Keyword::ROW]) {
                Ok(Some(WindowFrameExclusion::CurrentRow))
            } else if self.parse_keyword(Keyword::GROUP) {
                Ok(Some(WindowFrameExclusion::Group))
            } else if self.parse_keyword(Keyword::TIES) {
                Ok(Some(WindowFrameExclusion::Ties))
            } else if self.parse_keywords(&[Keyword::NO, Keyword::OTHERS]) {
                Ok(Some(WindowFrameExclusion::NoOthers))
            } else {
                let found = self.peek_token().cloned();
                self.expected("CURRENT ROW, GROUP, TIES or NO OTHERS", found)
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use usql_ast::types::*;

    use super::*;

    #[test]
    fn parse_from_clause() -> Result<(), ParserError> {
        Ok(())
    }

    #[test]
    fn parse_joined_table() -> Result<(), ParserError> {
        Ok(())
    }

    #[test]
    fn parse_join_specification() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "ON table1.id = table2.id")?.parse_join_spec()?,
            JoinSpec::On(Box::new(Expr::BinaryOp(BinaryOpExpr {
                left: Box::new(Expr::CompoundIdentifier(vec![
                    Ident::new("table1"),
                    Ident::new("id")
                ])),
                op: BinaryOperator::Equal,
                right: Box::new(Expr::CompoundIdentifier(vec![
                    Ident::new("table2"),
                    Ident::new("id")
                ])),
            })))
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "USING (id1)")?.parse_join_spec()?,
            JoinSpec::Using {
                columns: vec![Ident::new("id1")],
                alias: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "USING (id1, id2) AS ids")?.parse_join_spec()?,
            JoinSpec::Using {
                columns: vec![Ident::new("id1"), Ident::new("id2")],
                alias: Some(Ident::new("ids"))
            }
        );
        Ok(())
    }

    #[test]
    fn parse_where_clause() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "WHERE id = 1")?.parse_where_clause()?,
            Some(Where {
                expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                    left: Box::new(Expr::Identifier(Ident::new("id"))),
                    op: BinaryOperator::Equal,
                    right: Box::new(Expr::Literal(Literal::Number("1".into()))),
                }))
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "WHERE id IS NOT NULL")?.parse_where_clause()?,
            Some(Where {
                expr: Box::new(Expr::IsNull(IsNullExpr {
                    negated: true,
                    expr: Box::new(Expr::Identifier(Ident::new("id"))),
                }))
            })
        );
        Ok(())
    }

    #[test]
    fn parse_group_by_clause() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY ()")?.parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::Empty],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY DISTINCT id1")?.parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: Some(SetQuantifier::Distinct),
                list: vec![GroupingElement::OrdinarySet(GroupingSet::Column(
                    ObjectName::new(vec!["id1"])
                ))],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY (id1, id2)")?.parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::OrdinarySet(GroupingSet::Columns(vec![
                    ObjectName::new(vec!["id1"]),
                    ObjectName::new(vec!["id2"]),
                ]))],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY ROLLUP (id1, id2)")?
                .parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::Rollup(vec![GroupingSet::Columns(vec![
                    ObjectName::new(vec!["id1"]),
                    ObjectName::new(vec!["id2"]),
                ])])],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY CUBE (id1, id2)")?.parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::Cube(vec![GroupingSet::Columns(vec![
                    ObjectName::new(vec!["id1"]),
                    ObjectName::new(vec!["id2"]),
                ])])],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY GROUPING SETS (id1, id2)")?
                .parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::Sets(vec![
                    GroupingElement::OrdinarySet(GroupingSet::Column(ObjectName::new(vec!["id1"]))),
                    GroupingElement::OrdinarySet(GroupingSet::Column(ObjectName::new(vec!["id2"]))),
                ])],
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "GROUP BY GROUPING SETS ( (id1, id2) )")?
                .parse_group_by_clause()?,
            Some(GroupBy {
                quantifier: None,
                list: vec![GroupingElement::Sets(vec![GroupingElement::OrdinarySet(
                    GroupingSet::Columns(vec![
                        ObjectName::new(vec!["id1"]),
                        ObjectName::new(vec!["id2"]),
                    ])
                ),])],
            })
        );
        Ok(())
    }

    #[test]
    fn parse_having_clause() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "HAVING id = 1")?.parse_having_clause()?,
            Some(Having {
                expr: Box::new(Expr::BinaryOp(BinaryOpExpr {
                    left: Box::new(Expr::Identifier(Ident::new("id"))),
                    op: BinaryOperator::Equal,
                    right: Box::new(Expr::Literal(Literal::Number("1".into()))),
                }))
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "HAVING id IS NOT NULL")?.parse_having_clause()?,
            Some(Having {
                expr: Box::new(Expr::IsNull(IsNullExpr {
                    negated: true,
                    expr: Box::new(Expr::Identifier(Ident::new("id"))),
                }))
            })
        );
        Ok(())
    }

    #[test]
    fn parse_window_clause() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        let sql = "WINDOW w1 AS (PARTITION BY id1, id2 ORDER BY id1, id2), \
                                  w2 AS (PARTITION BY id2 ORDER BY id2 DESC NULLS LAST ROWS CURRENT ROW EXCLUDE NO OTHERS)";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_window_clause()?,
            Some(Window {
                list: vec![
                    WindowDef {
                        name: Ident::new("w1"),
                        spec: WindowSpec {
                            name: None,
                            partition_by: Some(vec![
                                ObjectName::new(vec!["id1"]),
                                ObjectName::new(vec!["id2"]),
                            ]),
                            order_by: Some(OrderBy {
                                list: vec![
                                    SortSpec {
                                        expr: Box::new(Expr::Identifier(Ident::new("id1"))),
                                        asc: None,
                                        nulls_first: None,
                                    },
                                    SortSpec {
                                        expr: Box::new(Expr::Identifier(Ident::new("id2"))),
                                        asc: None,
                                        nulls_first: None,
                                    }
                                ]
                            }),
                            window_frame: None,
                        }
                    },
                    WindowDef {
                        name: Ident::new("w2"),
                        spec: WindowSpec {
                            name: None,
                            partition_by: Some(vec![ObjectName::new(vec!["id2"])]),
                            order_by: Some(OrderBy {
                                list: vec![SortSpec {
                                    expr: Box::new(Expr::Identifier(Ident::new("id2"))),
                                    asc: Some(false),
                                    nulls_first: Some(false),
                                }]
                            }),
                            window_frame: Some(WindowFrame {
                                units: WindowFrameUnits::Rows,
                                start_bound: WindowFrameBound::CurrentRow,
                                end_bound: None,
                                exclusion: Some(WindowFrameExclusion::NoOthers),
                            }),
                        }
                    }
                ]
            })
        );
        Ok(())
    }

    #[test]
    fn parse_window_frame() -> Result<(), ParserError> {
        let dialect = usql_core::ansi::AnsiDialect::default();
        let sql = "ROWS CURRENT ROW";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_window_frame_clause()?,
            Some(WindowFrame {
                units: WindowFrameUnits::Rows,
                start_bound: WindowFrameBound::CurrentRow,
                end_bound: None,
                exclusion: None,
            })
        );
        let sql = "RANGE BETWEEN 1 PRECEDING AND 1 FOLLOWING EXCLUDE CURRENT ROW";
        assert_eq!(
            Parser::new_with_sql(&dialect, sql)?.parse_window_frame_clause()?,
            Some(WindowFrame {
                units: WindowFrameUnits::Range,
                start_bound: WindowFrameBound::Preceding(Some(1)),
                end_bound: Some(WindowFrameBound::Following(Some(1))),
                exclusion: Some(WindowFrameExclusion::CurrentRow),
            })
        );
        Ok(())
    }
}
