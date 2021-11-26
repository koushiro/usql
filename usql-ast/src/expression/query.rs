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
    /// `ORDER BY { <sort_key> [ ASC | DESC ] [ NULLS FIRST | NULLS LAST ] } [, ...]`
    pub order_by: Option<OrderBy>,
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
        if let Some(order_by) = &self.order_by {
            write!(f, " {}", order_by)?;
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

// ============================================================================
// with clause
// ============================================================================

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

// ============================================================================
// order by clause
// ============================================================================

/// `ORDER BY` clause.
///
/// ```txt
/// ORDER BY <sort specification>  [ { ,  <sort specification>  }... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderBy {
    /// The sort specification list.
    pub list: Vec<SortSpec>,
}

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ORDER BY {}", display_comma_separated(&self.list))
    }
}

/// A sort specification.
///
/// ```txt
/// <sort_key>  [ ASC | DESC  ] [ NULLS FIRST | NULLS LAST  ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SortSpec {
    /// Sort key
    pub expr: Box<Expr>,
    /// Optional `ASC` or `DESC`
    pub asc: Option<bool>,
    /// Optional `NULLS FIRST` or `NULLS LAST`
    pub nulls_first: Option<bool>,
}

impl fmt::Display for SortSpec {
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

// ============================================================================
// limit clause (Not ANSI SQL standard, but most dialects support it)
// ============================================================================

/// Limit clause.
///
/// ```txt
/// LIMIT [ offset, ] row_count
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Limit {
    pub offset: Option<Literal>,
    pub count: Literal,
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

// ============================================================================
// result offset clause
// ============================================================================

/// Offset clause.
///
/// ```txt
/// OFFSET <offset> [ { ROW | ROWS } ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset {
    pub offset: Literal,
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

// ============================================================================
// fetch first clause
// ============================================================================

/// Fetch first clause.
///
/// ```txt
/// FETCH { FIRST | NEXT } <row> [ PERCENT ] { ROW | ROWS } | { ONLY | WITH TIES }
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fetch {
    pub quantity: Option<Literal>,
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
