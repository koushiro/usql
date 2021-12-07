mod function;
mod query;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, vec, vec::Vec};

use usql_ast::{expression::*, types::*};
use usql_core::{Dialect, Keyword};
use usql_lexer::{Token, Word};

use crate::{
    error::{parse_error, ParserError},
    parser::Parser,
};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parses a new expression.
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_subexpr(0)
    }

    /// Parses tokens until the precedence changes.
    pub fn parse_subexpr(&mut self, precedence: u8) -> Result<Expr, ParserError> {
        let mut expr = self.parse_prefix()?;
        loop {
            let next_precedence = self.next_precedence()?;
            if precedence >= next_precedence {
                break;
            }
            expr = self.parse_infix(Box::new(expr), next_precedence)?;
        }
        Ok(expr)
    }

    const UNARY_NOT_PREC: u8 = 15;
    const BETWEEN_PREC: u8 = 20;
    const PLUS_MINUS_PREC: u8 = 30;

    /// Parses an expression prefix.
    pub fn parse_prefix(&mut self) -> Result<Expr, ParserError> {
        let token = self.peek_next_token().cloned();
        if let Some(token) = token {
            match token {
                Token::Number(_)
                | Token::String(_)
                | Token::NationalString(_)
                | Token::HexString(_)
                | Token::BitString(_) => Ok(Expr::Literal(self.parse_literal()?)),
                Token::Word(word) => match word.keyword {
                    Some(Keyword::NULL)
                    | Some(Keyword::TRUE)
                    | Some(Keyword::FALSE)
                    | Some(Keyword::DATE)
                    | Some(Keyword::TIME)
                    | Some(Keyword::DATETIME)
                    | Some(Keyword::INTERVAL) => Ok(Expr::Literal(self.parse_literal()?)),
                    Some(Keyword::NOT) => {
                        self.next_token(); // consume the `NOT` keyword
                        Ok(Expr::UnaryOp(UnaryOpExpr {
                            op: UnaryOperator::Not,
                            expr: Box::new(self.parse_subexpr(Self::UNARY_NOT_PREC)?),
                        }))
                    }
                    // Keyword::CASE => self.parse_case_expr(),
                    // Keyword::CAST => self.parse_cast_expr(),
                    // Keyword::EXISTS => self.parse_exists_expr(),
                    // Keyword::EXTRACT => self.parse_extract_expr(),
                    // Keyword::SUBSTRING => self.parse_substring_expr(),
                    // Keyword::TRIM => self.parse_trim_expr(),
                    // Keyword::LISTAGG => self.parse_listagg_expr(),
                    _ if self.peek_next_token() == Some(&Token::Period) => {
                        self.next_token(); // consume the token word.
                        let mut id_parts: Vec<Ident> = vec![Ident {
                            value: word.value,
                            quote: word.quote,
                        }];
                        let mut ends_with_wildcard = false;
                        while self.next_token_if_is(&Token::Period) {
                            match self.next_token() {
                                Some(Token::Word(w)) => id_parts.push(Ident {
                                    value: w.value,
                                    quote: w.quote,
                                }),
                                Some(Token::Asterisk) => {
                                    ends_with_wildcard = true;
                                    break;
                                }
                                unexpected => {
                                    return self
                                        .expected("an identifier or a '*' after '.'", unexpected)
                                }
                            }
                        }
                        if ends_with_wildcard {
                            Ok(Expr::QualifiedWildcard(id_parts))
                        } else {
                            Ok(Expr::CompoundIdentifier(id_parts))
                        }
                    }
                    _ => Ok(Expr::Identifier(self.parse_identifier()?)),
                },
                Token::Minus => {
                    self.next_token(); // consume `-`
                    Ok(Expr::UnaryOp(UnaryOpExpr {
                        op: UnaryOperator::Minus,
                        expr: Box::new(self.parse_subexpr(Self::PLUS_MINUS_PREC)?),
                    }))
                }
                Token::Plus => {
                    self.next_token(); // consume `+`
                    Ok(Expr::UnaryOp(UnaryOpExpr {
                        op: UnaryOperator::Plus,
                        expr: Box::new(self.parse_subexpr(Self::PLUS_MINUS_PREC)?),
                    }))
                }
                Token::Asterisk => {
                    self.next_token(); // consume `*`
                    Ok(Expr::Wildcard)
                }
                Token::LeftParen => {
                    self.next_token(); // consume `(`
                    let expr = if self.next_is_query() {
                        Expr::Subquery(Box::new(self.parse_query_expr(true)?))
                    } else {
                        Expr::Nested(Box::new(self.parse_expr()?))
                    };
                    self.expect_token(&Token::RightParen)?;
                    Ok(expr)
                }
                unexpected => self.expected("an expression infix", Some(unexpected)),
            }
        } else {
            self.expected("an expression prefix", Option::<Token>::None)
        }
    }

    /// Gets the precedence of the next token.
    pub fn next_precedence(&mut self) -> Result<u8, ParserError> {
        let precedence = if let Some(token) = self.peek_next_token() {
            match token {
                token if token.is_keyword(Keyword::OR) => Ok(5),
                token if token.is_keyword(Keyword::AND) => Ok(10),
                token if token.is_keyword(Keyword::XOR) => Ok(24),
                Token::Word(w) if w.keyword == Some(Keyword::NOT) => match self.peek_next_token() {
                    // The precedence of NOT varies depending on keyword that
                    // follows it. If it is followed by IN, BETWEEN, or LIKE,
                    // it takes on the precedence of those tokens. Otherwise it
                    // is not an infix operator, and therefore has zero precedence.
                    Some(token) if token.is_keyword(Keyword::IN) => Ok(Self::BETWEEN_PREC),
                    Some(token) if token.is_keyword(Keyword::BETWEEN) => Ok(Self::BETWEEN_PREC),
                    Some(token) if token.is_keyword(Keyword::LIKE) => Ok(Self::BETWEEN_PREC),
                    Some(token) if token.is_keyword(Keyword::ILIKE) => Ok(Self::BETWEEN_PREC),
                    _ => Ok(0),
                },
                token if token.is_keyword(Keyword::IS) => Ok(17),
                token if token.is_keyword(Keyword::IN) => Ok(Self::BETWEEN_PREC),
                token if token.is_keyword(Keyword::BETWEEN) => Ok(Self::BETWEEN_PREC),
                token if token.is_keyword(Keyword::LIKE) => Ok(Self::BETWEEN_PREC),
                token if token.is_keyword(Keyword::ILIKE) => Ok(Self::BETWEEN_PREC),
                Token::Equal
                | Token::Less
                | Token::LessOrEqual
                | Token::NotEqual
                | Token::Greater
                | Token::GreaterOrEqual
                | Token::Tilde => Ok(20),
                Token::Pipe => Ok(21),
                Token::Caret | Token::Sharp | Token::LeftShift | Token::RightShift => Ok(22),
                Token::Ampersand => Ok(23),
                Token::Plus | Token::Minus => Ok(Self::PLUS_MINUS_PREC),
                Token::Asterisk | Token::Slash | Token::Percent | Token::Concat => Ok(40),
                Token::DoubleColon => Ok(50),
                Token::Exclamation => Ok(50),
                Token::LeftBracket | Token::RightBracket => Ok(10),
                _ => Ok(0),
            }
        } else {
            Ok(0)
        };
        self.reset_peek_cursor();
        precedence
    }

    /// Parses an operator following an expression.
    pub fn parse_infix(&mut self, expr: Box<Expr>, precedence: u8) -> Result<Expr, ParserError> {
        let token = self.next_token();
        if let Some(token) = &token {
            let regular_binary_operator = match token {
                Token::Plus => Some(BinaryOperator::Plus),
                Token::Minus => Some(BinaryOperator::Minus),
                Token::Asterisk => Some(BinaryOperator::Multiply),
                Token::Slash => Some(BinaryOperator::Divide),
                Token::Percent => Some(BinaryOperator::Modulo),

                Token::Greater => Some(BinaryOperator::Greater),
                Token::Less => Some(BinaryOperator::Less),
                Token::GreaterOrEqual => Some(BinaryOperator::GreaterOrEqual),
                Token::LessOrEqual => Some(BinaryOperator::LessOrEqual),
                Token::Equal => Some(BinaryOperator::Equal),
                Token::NotEqual => Some(BinaryOperator::NotEqual),

                Token::Concat => Some(BinaryOperator::StringConcat),

                Token::Ampersand => Some(BinaryOperator::BitwiseAnd),
                Token::Pipe => Some(BinaryOperator::BitwiseOr),
                Token::Caret => Some(BinaryOperator::BitwiseXor),
                Token::LeftShift => Some(BinaryOperator::BitwiseLeftShift),
                Token::RightShift => Some(BinaryOperator::BitwiseRightShift),

                Token::Word(word) => match word.keyword {
                    Some(Keyword::AND) => Some(BinaryOperator::And),
                    Some(Keyword::OR) => Some(BinaryOperator::Or),
                    Some(Keyword::XOR) => Some(BinaryOperator::Xor),
                    Some(Keyword::LIKE) => Some(BinaryOperator::Like),
                    Some(Keyword::ILIKE) => Some(BinaryOperator::ILike),
                    Some(Keyword::NOT) if self.parse_keyword(Keyword::LIKE) => {
                        Some(BinaryOperator::NotLike)
                    }
                    Some(Keyword::NOT) if self.parse_keyword(Keyword::ILIKE) => {
                        Some(BinaryOperator::NotILike)
                    }
                    _ => None,
                },
                _ => None,
            };

            if let Some(op) = regular_binary_operator {
                let right = self.parse_subexpr(precedence)?;
                Ok(Expr::BinaryOp(BinaryOpExpr {
                    left: expr,
                    op,
                    right: Box::new(right),
                }))
            } else if let Token::Word(Word {
                keyword: Some(keyword),
                ..
            }) = token
            {
                match keyword {
                    Keyword::IS => {
                        let negated = self.parse_keyword(Keyword::NOT);
                        if self.parse_keyword(Keyword::NULL) {
                            Ok(Expr::IsNull(IsNullExpr { negated, expr }))
                        } else if self.parse_keywords(&[Keyword::DISTINCT, Keyword::FROM]) {
                            Ok(Expr::IsDistinctFrom(IsDistinctFromExpr {
                                negated,
                                left: expr,
                                right: Box::new(self.parse_expr()?),
                            }))
                        } else {
                            let found = self.peek_token().cloned();
                            self.expected("[NOT] NULL or [NOT] DISTINCT FROM after IS", found)
                        }
                    }
                    Keyword::NOT => {
                        if self.parse_keyword(Keyword::IN) {
                            self.parse_in(expr, true)
                        } else if self.parse_keyword(Keyword::BETWEEN) {
                            self.parse_between(expr, true)
                        } else {
                            let found = self.peek_token().cloned();
                            self.expected("[NOT] IN or [NOT] BETWEEN after NOT", found)
                        }
                    }
                    Keyword::IN => self.parse_in(expr, false),
                    Keyword::BETWEEN => self.parse_between(expr, false),
                    // Can only happen if `next_precedence` got out of sync with this function
                    _ => parse_error(format!("No infix parser for token {:?}", token)),
                }
            } else {
                self.expected("expression infix", Some(token))
            }
        } else {
            self.expected("expression infix", Option::<Token>::None)
        }
    }

    /// Parses the parens following the `[ NOT ] IN (...)` operator,
    /// assuming the `[NOT] IN` keyword have already been consumed.
    fn parse_in(&mut self, expr: Box<Expr>, negated: bool) -> Result<Expr, ParserError> {
        self.expect_token(&Token::LeftParen)?;
        let in_op = if self.next_is_query() {
            // don't consume the `SELECT` or `WITH` keyword.
            Expr::InSubquery(InSubqueryExpr {
                expr,
                negated,
                subquery: Box::new(self.parse_query_expr(true)?),
            })
        } else {
            Expr::InList(InListExpr {
                expr,
                negated,
                list: self.parse_comma_separated(Parser::parse_expr)?,
            })
        };
        self.expect_token(&Token::RightParen)?;
        Ok(in_op)
    }

    /// Parses `[NOT] BETWEEN <low> AND <high>`,
    /// assuming the `[NOT] BETWEEN` keyword have already been consumed.
    fn parse_between(&mut self, expr: Box<Expr>, negated: bool) -> Result<Expr, ParserError> {
        // Stop parsing subexpressions for <low> and <high> on tokens with
        // precedence lower than that of `BETWEEN`, such as `AND`, `IS`, etc.
        let low = self.parse_subexpr(Self::BETWEEN_PREC)?;
        self.expect_keyword(Keyword::AND)?;
        let high = self.parse_subexpr(Self::BETWEEN_PREC)?;
        Ok(Expr::Between(BetweenExpr {
            expr,
            negated,
            low: Box::new(low),
            high: Box::new(high),
        }))
    }

    fn next_is_query(&mut self) -> bool {
        self.peek_token()
            .map(|token| token.is_one_of_keywords(&[Keyword::SELECT, Keyword::WITH]))
            .flatten()
            .is_some()
    }
}
