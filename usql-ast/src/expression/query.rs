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
    /// SELECT or UNION / EXCEPT / INTERSECT
    pub body: QueryBody,
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
        write!(f, "{}", self.body)?;
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

/// The body of query expression.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryBody {
    Select(Box<Select>),
    Operation {
        left: Box<QueryBody>,
        op: QueryBodyOperator,
        right: Box<QueryBody>,
    },
}

impl fmt::Display for QueryBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryBody::Select(select) => write!(f, "{}", select),
            QueryBody::Operation { left, op, right } => write!(f, "{} {} {}", left, op, right),
        }
    }
}

/// The operators that can be used in the query expression body.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryBodyOperator {
    Union(Option<SetQuantifier>),
    Except(Option<SetQuantifier>),
    Intersect(Option<SetQuantifier>),
}

impl fmt::Display for QueryBodyOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            QueryBodyOperator::Union(None) => "UNION",
            QueryBodyOperator::Union(Some(SetQuantifier::All)) => "UNION ALL",
            QueryBodyOperator::Union(Some(SetQuantifier::Distinct)) => "UNION DISTINCT",
            QueryBodyOperator::Except(None) => "EXCEPT",
            QueryBodyOperator::Except(Some(SetQuantifier::All)) => "EXCEPT ALL",
            QueryBodyOperator::Except(Some(SetQuantifier::Distinct)) => "EXCEPT DISTINCT",
            QueryBodyOperator::Intersect(None) => "INTERSECT",
            QueryBodyOperator::Intersect(Some(SetQuantifier::All)) => "INTERSECT ALL",
            QueryBodyOperator::Intersect(Some(SetQuantifier::Distinct)) => "INTERSECT DISTINCT",
        })
    }
}

/// The option of query body operator.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SetQuantifier {
    All,
    Distinct,
}

impl fmt::Display for SetQuantifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::All => "ALL",
            Self::Distinct => "DISTINCT",
        })
    }
}

/// A restricted variant of `SELECT` (without CTEs/`ORDER BY`), which may
/// appear either as the only body item of an `Query`, or as an operand to a
/// set operation like `UNION`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Select {
    /// Set quantifier
    pub quantifier: Option<SetQuantifier>,
    /// projection expressions
    pub projection: Vec<SelectItem>,

    /// FROM clause
    pub from: Vec<TableWithJoins>,
    /// WHERE clause
    pub selection: Option<Box<Expr>>,
    /// GROUP BY clause
    pub group_by: Vec<Expr>,
    /// HAVING clause
    pub having: Option<Box<Expr>>,
    /// WINDOW clause
    pub windows: Option<Vec<Window>>,
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SELECT")?;
        if let Some(quantifier) = &self.quantifier {
            write!(f, " {}", quantifier)?;
        }
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
        if let Some(windows) = &self.windows {
            write!(f, " WINDOW {}", display_comma_separated(windows))?;
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
    /// An expression, maybe followed by `[ AS ] alias`
    #[doc(hidden)]
    DerivedColumn {
        expr: Box<Expr>,
        alias: Option<Ident>,
    },
}

impl fmt::Display for SelectItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectItem::Wildcard => write!(f, "*"),
            SelectItem::QualifiedWildcard(prefix) => write!(f, "{}.*", prefix),
            SelectItem::DerivedColumn { expr, alias } => {
                if let Some(alias) = alias {
                    write!(f, "{} AS {}", expr, alias)
                } else {
                    write!(f, "{}", expr)
                }
            }
        }
    }
}

/// From clause.
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
            write!(f, " {}", join)?;
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
        /// Table or query name.
        name: ObjectName,
        alias: Option<TableAlias>,
    },
    Derived {
        lateral: bool,
        subquery: Box<Query>,
        alias: Option<TableAlias>,
    },
    /// Represents a parenthesized joined table.
    /// The SQL spec only allows a join expression
    /// (`(foo <JOIN> bar [ <JOIN> baz ... ])`) to be nested, possibly several times.
    NestedJoin(Box<TableWithJoins>),
}

impl fmt::Display for TableFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Table { name, alias } => {
                write!(f, "{}", name)?;
                if let Some(alias) = alias {
                    write!(f, " AS {}", alias)?;
                }
                Ok(())
            }
            Self::Derived {
                lateral,
                subquery,
                alias,
            } => {
                if *lateral {
                    write!(f, "LATERAL ")?;
                }
                write!(f, "({})", subquery)?;
                if let Some(alias) = alias {
                    write!(f, " AS {}", alias)?;
                }
                Ok(())
            }
            Self::NestedJoin(table) => write!(f, "({})", table),
        }
    }
}

/// Table alias.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableAlias {
    /// Alias name.
    pub name: Ident,
    /// Columns.
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

/// The `JOIN` relation.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Join {
    pub join: JoinOperator,
    pub relation: TableFactor,
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.join {
            JoinOperator::CrossJoin => write!(f, "CROSS JOIN {}", self.relation),
            JoinOperator::Inner(constraint) => write!(
                f,
                "{}JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::LeftOuter(constraint) => write!(
                f,
                "{}LEFT JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::RightOuter(constraint) => write!(
                f,
                "{}RIGHT JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
            JoinOperator::FullOuter(constraint) => write!(
                f,
                "{}FULL JOIN {}{}",
                constraint.prefix(),
                self.relation,
                constraint.suffix(),
            ),
        }
    }
}

/// The join operator.
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

/// The constraint of join operator.
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
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct With {
    #[doc(hidden)]
    pub recursive: bool,
    /// Common table expressions.
    pub ctes: Vec<Cte>,
}

impl fmt::Display for With {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WITH {}{}",
            if self.recursive { "RECURSIVE " } else { "" },
            display_comma_separated(&self.ctes)
        )
    }
}

/// A single CTE (used after `WITH`): `alias [(col1, col2, ...)] AS ( query )`.
/// The names in the column list before `AS`, when specified, replace the names
/// of the columns returned by the query.
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

/// `ORDER BY` clause.
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

/// Stores the keyword after `OFFSET <number>`.
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

/// Window clause.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Window {
    /// New window name.
    pub name: Ident,
    /// Window specification.
    pub spec: WindowSpec,
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} AS ({})", self.name, self.spec)
    }
}
