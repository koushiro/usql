#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::{expression::*, types::*, utils::display_comma_separated};

/// The most complete variant of a `SELECT` query expression, optionally
/// including `WITH`, `UNION` / other set operations, and `ORDER BY`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Query {
    /// WITH (common table expressions, or CTEs)
    pub with: Option<With>,
    // /// SELECT or UNION / EXCEPT / INTERSECT
    // pub body: QueryBody,
    /// `ORDER BY <expr> [ ASC | DESC ] [ NULLS { FIRST | LAST } ] [, ...]`
    pub order_by: Vec<OrderBy>,
    /// `LIMIT { <N> | ALL }`
    pub limit: Option<Limit>,
    /// `OFFSET <N> [ { ROW | ROWS } ]`
    pub offset: Option<Offset>,
    /// `FETCH { FIRST | NEXT } <N> [ PERCENT ] { ROW | ROWS } | { ONLY | WITH TIES }`
    pub fetch: Option<Fetch>,
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(with) = &self.with {
            write!(f, "{} ", with)?;
        }
        // write!(f, "{}", self.body)?;
        if !self.order_by.is_empty() {
            write!(f, " ORDER BY {}", display_comma_separated(&self.order_by))?;
        }
        if let Some(limit) = &self.limit {
            write!(f, " {}", limit)?;
        }
        if let Some(offset) = &self.offset {
            write!(f, " {}", offset)?;
        }
        if let Some(fetch) = &self.fetch {
            write!(f, " {}", fetch)?;
        }
        Ok(())
    }
}

/// A restricted variant of `SELECT` (without CTEs/`ORDER BY`), which may
/// appear either as the only body item of an `Query`, or as an operand to a
/// set operation like `UNION`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Select {
    #[doc(hidden)]
    pub distinct: bool,
    /// projection expressions
    pub projection: Vec<SelectItem>,
    /// FROM
    pub from: Vec<TableWithJoins>,
    /// WHERE
    pub selection: Option<Expr>,
    /// GROUP BY
    pub group_by: Vec<Expr>,
    /// HAVING
    pub having: Option<Expr>,
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SELECT{}", if self.distinct { " DISTINCT" } else { "" })?;
        write!(f, " {}", display_comma_separated(&self.projection))?;
        if !self.from.is_empty() {
            write!(f, " FROM {}", display_comma_separated(&self.from))?;
        }
        if let Some(selection) = &self.selection {
            write!(f, " WHERE {}", selection)?;
        }
        if !self.group_by.is_empty() {
            write!(f, " GROUP BY {}", display_comma_separated(&self.group_by))?;
        }
        if let Some(having) = &self.having {
            write!(f, " HAVING {}", having)?;
        }
        Ok(())
    }
}

/// One item of the comma-separated list following `SELECT`
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectItem {
    /// An unqualified `*`
    Wildcard,
    /// `alias.*` or even `schema.table.*`
    QualifiedWildcard(ObjectName),
    /// Any expression, not followed by `[ AS ] alias`
    UnnamedExpr(Expr),
    /// An expression, followed by `[ AS ] alias`
    #[doc(hidden)]
    ExprWithAlias { expr: Expr, alias: Ident },
}

impl fmt::Display for SelectItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectItem::Wildcard => write!(f, "*"),
            SelectItem::QualifiedWildcard(prefix) => write!(f, "{}.*", prefix),
            SelectItem::UnnamedExpr(expr) => write!(f, "{}", expr),
            SelectItem::ExprWithAlias { expr, alias } => write!(f, "{} AS {}", expr, alias),
        }
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableWithJoins {
    pub relation: TableFactor,
    pub joins: Vec<Join>,
}

impl fmt::Display for TableWithJoins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.relation)?;
        for join in &self.joins {
            write!(f, "{}", join)?;
        }
        Ok(())
    }
}

/// A table name or a parenthesized subquery with an optional alias
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableFactor {
    Table {
        name: ObjectName,
        alias: Option<TableAlias>,
        /// Arguments of a table-valued function, as supported by Postgres
        /// and MSSQL. Note that deprecated MSSQL `FROM foo (NOLOCK)` syntax
        /// will also be parsed as `args`.
        args: Vec<FunctionArg>,
        /// MSSQL-specific `WITH (...)` hints such as NOLOCK.
        with_hints: Vec<Expr>,
    },
    Derived {
        lateral: bool,
        subquery: Box<Query>,
        alias: Option<TableAlias>,
    },
    /// `TABLE(<expr>)[ AS <alias> ]`
    TableFunction {
        expr: Expr,
        alias: Option<TableAlias>,
    },
    /// Represents a parenthesized table factor. The SQL spec only allows a
    /// join expression (`(foo <JOIN> bar [ <JOIN> baz ... ])`) to be nested,
    /// possibly several times.
    ///
    /// The parser may also accept non-standard nesting of bare tables for some
    /// dialects, but the information about such nesting is stripped from AST.
    NestedJoin(Box<TableWithJoins>),
}

impl fmt::Display for TableFactor {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableAlias {
    pub name: Ident,
    pub columns: Vec<Ident>,
}

impl fmt::Display for TableAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.columns.is_empty() {
            write!(f, " ({})", display_comma_separated(&self.columns))?;
        }
        Ok(())
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join {
    pub relation: TableFactor,
    pub join_operator: JoinOperator,
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.join_operator {
            JoinOperator::Inner(constraint) => write!(
                f,
                " {}JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::LeftOuter(constraint) => write!(
                f,
                " {}LEFT JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::RightOuter(constraint) => write!(
                f,
                " {}RIGHT JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::FullOuter(constraint) => write!(
                f,
                " {}FULL JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::CrossJoin => write!(f, " CROSS JOIN {}", self.relation),
        }
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinOperator {
    CrossJoin,
    Inner(JoinConstraint),
    LeftOuter(JoinConstraint),
    RightOuter(JoinConstraint),
    FullOuter(JoinConstraint),
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinConstraint {
    On(Expr),
    Using(Vec<Ident>),
    Natural,
    None,
}

impl JoinConstraint {
    fn prefix(&self) -> &'static str {
        match self {
            Self::Natural => "NATURAL ",
            _ => "",
        }
    }

    fn suffix(&self) -> impl fmt::Display + '_ {
        struct Suffix<'a>(&'a JoinConstraint);
        impl<'a> fmt::Display for Suffix<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self.0 {
                    JoinConstraint::On(expr) => write!(f, " ON {}", expr),
                    JoinConstraint::Using(attrs) => {
                        write!(f, " USING({})", display_comma_separated(attrs))
                    }
                    _ => Ok(()),
                }
            }
        }
        Suffix(self)
    }
}

// ================================================================================================
// Optional clause
// ================================================================================================

/// With clause.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct With {
    pub recursive: bool,
    pub cte_tables: Vec<Cte>,
}

impl fmt::Display for With {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WITH {}{}",
            if self.recursive { "RECURSIVE " } else { "" },
            display_comma_separated(&self.cte_tables)
        )
    }
}

/// A single CTE (used after `WITH`): `alias [(col1, col2, ...)] AS ( query )`
/// The names in the column list before `AS`, when specified, replace the names
/// of the columns returned by the query. The parser does not validate that the
/// number of columns in the query matches the number of columns in the query.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cte {
    pub alias: TableAlias,
    pub query: Query,
    pub from: Option<Ident>,
}

impl fmt::Display for Cte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} AS ({})", self.alias, self.query)?;
        if let Some(from) = &self.from {
            write!(f, " FROM {}", from)?;
        }
        Ok(())
    }
}

/// `ORDER BY` clause
///
/// ```txt
/// <expr> [ASC | DESC] [NULLS FIRST | NULLS LAST]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderBy {
    /// Order by expression
    pub expr: Box<Expr>,
    /// Optional `ASC` or `DESC`
    pub asc: Option<bool>,
    /// Optional `NULLS FIRST` or `NULLS LAST`
    pub nulls_first: Option<bool>,
}

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expr)?;
        match self.asc {
            Some(true) => write!(f, " ASC")?,
            Some(false) => write!(f, " DESC")?,
            None => (),
        }
        match self.nulls_first {
            Some(true) => write!(f, " NULLS FIRST")?,
            Some(false) => write!(f, " NULLS LAST")?,
            None => (),
        }
        Ok(())
    }
}

/// Limit clause (Not-standard).
///
/// ```txt
/// LIMIT [ offset, ] row_count
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Limit {
    pub offset: Option<Expr>,
    pub count: Expr,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(offset) = &self.offset {
            write!(f, "LIMIT {},{}", offset, self.count)
        } else {
            write!(f, "LIMIT {}", self.count)
        }
    }
}

/// Offset clause.
///
/// ```txt
/// OFFSET <offset> [ { ROW | ROWS } ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset {
    pub offset: Expr,
    pub rows: OffsetRows,
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OFFSET {}{}", self.offset, self.rows)
    }
}

/// Stores the keyword after `OFFSET <number>`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OffsetRows {
    /// Omitting ROW/ROWS is non-standard MySQL quirk.
    None,
    Row,
    Rows,
}

impl fmt::Display for OffsetRows {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OffsetRows::None => Ok(()),
            OffsetRows::Row => f.write_str(" ROW"),
            OffsetRows::Rows => f.write_str(" ROWS"),
        }
    }
}

/// Fetch first clause.
///
/// ```txt
/// FETCH { FIRST | NEXT } <row> [ PERCENT ] { ROW | ROWS } | { ONLY | WITH TIES }
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fetch {
    pub quantity: Option<Expr>,
    /// Flag indicates that if the quantity is percentage.
    pub percent: bool,
    pub with_ties: bool,
}

impl fmt::Display for Fetch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let extension = if self.with_ties { "WITH TIES" } else { "ONLY" };
        if let Some(ref quantity) = self.quantity {
            let percent = if self.percent { " PERCENT" } else { "" };
            write!(f, "FETCH FIRST {}{} ROWS {}", quantity, percent, extension)
        } else {
            write!(f, "FETCH FIRST ROWS {}", extension)
        }
    }
}
