// table expression
mod table;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

pub use self::table::*;
use crate::{expression::*, types::*, utils::display_comma_separated};

/// The most complete variant of a `SELECT` query expression, optionally
/// including `WITH`, `UNION` / other set operations, and `ORDER BY`.
///
/// ```txt
/// <query expression> ::= [ <with clause> ] <query expression body>
///     [ <order by clause> ] [ <result offset clause> ] [ <fetch first clause> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Query {
    /// WITH (common table expressions, or CTEs)
    pub with: Option<With>,
    /// SELECT or UNION / EXCEPT / INTERSECT
    pub body: QueryBody,
    /// `ORDER BY { <sort_key> [ ASC | DESC ] [ NULLS FIRST | NULLS LAST ] } [, ...]`
    pub order_by: Option<OrderBy>,
    /// `OFFSET <N> [ { ROW | ROWS } ]`
    pub offset: Option<Offset>,
    /// `LIMIT { <N> | ALL }`
    pub limit: Option<Limit>,
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
/// <table value constructor> ::= VALUES <row value expression> [ { , <row value expression> }... ]
/// <explicit table> ::= TABLE <table or query name>
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryBody {
    /// Query specification, like `SELECT ... FROM ... GROUP BY ... HAVING ... WINDOW ...`
    QuerySpec(Box<QuerySpec>),
    /// Parenthesized (non-with clause) subquery expression
    Subquery(Box<Query>),
    // Table value constructor
    Values(Values),
    /// Explicit table
    Table(ObjectName),
    /// UNION/EXCEPT/INTERSECT operation of two query bodies
    Operation {
        left: Box<QueryBody>,
        op: QueryBodyOperator,
        quantifier: Option<SetQuantifier>,
        right: Box<QueryBody>,
    },
}

impl fmt::Display for QueryBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::QuerySpec(select) => write!(f, "{}", select),
            Self::Subquery(query) => write!(f, "({})", query),
            Self::Values(values) => write!(f, "{}", values),
            Self::Table(name) => write!(f, "{}", name),
            Self::Operation {
                left,
                op,
                quantifier,
                right,
            } => {
                write!(f, "{} {}", left, op)?;
                if let Some(quantifier) = quantifier {
                    write!(f, " {}", quantifier)?;
                }
                write!(f, " {}", right)
            }
        }
    }
}

/// The values list, which provides a way to generate a “constant table” that can be used in a query.
///
/// ```txt
/// <table value constructor> ::= VALUES <row value expression> [ { , <row value expression> }... ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Values {
    /// The list of row value expression.
    pub list: Vec<Vec<Expr>>,
}

impl fmt::Display for Values {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("VALUES ")?;
        let mut delim = "";
        for row in &self.list {
            write!(f, "{}", delim)?;
            delim = ", ";
            write!(f, "({})", display_comma_separated(row))?;
        }
        Ok(())
    }
}

/// The operators that can be used in the query expression body.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryBodyOperator {
    Union,
    Except,
    Intersect,
}

impl fmt::Display for QueryBodyOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Union => "UNION",
            Self::Except => "EXCEPT",
            Self::Intersect => "INTERSECT",
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
///
/// ```txt
/// <with clause> ::= WITH [ RECURSIVE ] <with list>
/// <with list> ::= <with list element> [ { , <with list element> }... ]
/// <with list element> ::= <query name> [ ( <column list> ) ] AS ( <query expression> )
/// ```
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
    pub query: Box<Query>,
}

impl fmt::Display for Cte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} AS ({})", self.alias, self.query)
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
/// <sort key>  [ ASC | DESC  ] [ NULLS FIRST | NULLS LAST  ]
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
/// NOTE: we don't support `LIMIT [ offset, ] row_count` syntax yet.
///
/// ```txt
/// LIMIT <count>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Limit {
    /// The row count.
    pub count: Literal,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LIMIT {}", self.count)
    }
}

// ============================================================================
// result offset clause
// ============================================================================

/// Offset clause.
///
/// ```txt
/// OFFSET <count> [ { ROW | ROWS } ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset {
    pub count: Literal,
    pub rows: OffsetRows,
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OFFSET {}{}", self.count, self.rows)
    }
}

/// Stores the keyword after `OFFSET <number>`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OffsetRows {
    Row,
    Rows,
    /// Omitting ROW/ROWS is non-standard MySQL quirk.
    None,
}

impl fmt::Display for OffsetRows {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OffsetRows::Row => f.write_str(" ROW"),
            OffsetRows::Rows => f.write_str(" ROWS"),
            OffsetRows::None => Ok(()),
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
