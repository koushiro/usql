mod function;
mod query;

use usql_ast::expression::*;
use usql_core::Dialect;

use crate::{error::ParserError, parser::Parser};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parse a new expression.
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_subexpr(0)
    }

    /// Parse tokens until the precedence changes.
    pub fn parse_subexpr(&mut self, precedence: u8) -> Result<Expr, ParserError> {
        // log::debug!("parsing expr");
        let mut expr = self.parse_expr_prefix()?;
        // log::debug!("prefix: {:?}", expr);
        loop {
            let next_precedence = self.next_precedence()?;
            // log::debug!("next precedence: {:?}", next_precedence);
            if precedence >= next_precedence {
                break;
            }
            expr = self.parse_expr_infix(expr, next_precedence)?;
        }
        Ok(expr)
    }

    /// Get the precedence of the next token.
    pub fn next_precedence(&self) -> Result<u8, ParserError> {
        todo!()
    }

    /// Parse an expression prefix.
    pub fn parse_expr_prefix(&mut self) -> Result<Expr, ParserError> {
        todo!()
    }

    /// Parse an operator following an expression.
    pub fn parse_expr_infix(&mut self, _expr: Expr, _precedence: u8) -> Result<Expr, ParserError> {
        todo!()
    }
}
