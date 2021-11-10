mod function;
mod operator;
mod query;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use self::{
    function::*,
    operator::{BinaryOperator, UnaryOperator},
    query::*,
};
use crate::{
    types::{DataType, DateTimeField, Ident, Literal, ObjectName},
    utils::{display_comma_separated, display_separated, escape_single_quote_string},
};

/// SQL expression type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Expr {
    /// A literal value, such as string, number, date.
    Literal(Literal),

    /// A constant of form `<data_type> 'value'`.
    /// This can represent ANSI SQL `DATE`, `TIME`, and `TIMESTAMP` literals (such as `DATE '2020-01-01'`),
    /// as well as constants of other types (a non-standard PostgreSQL extension).
    #[doc(hidden)]
    TypedString { data_type: DataType, value: String },

    /// Identifier e.g. table name or column name
    Identifier(Ident),

    /// Unqualified wildcard (`*`). SQL allows this in limited contexts, such as:
    /// - right after `SELECT` (which is represented as a [SelectItem::Wildcard] instead)
    /// - or as part of an aggregate function, e.g. `COUNT(*)`,
    ///
    /// ...but we currently also accept it in contexts where it doesn't make
    /// sense, such as `* + *`
    Wildcard,
    /// Qualified wildcard, e.g. `alias.*` or `schema.table.*`.
    /// (Same caveats apply to `QualifiedWildcard` as to `Wildcard`.)
    QualifiedWildcard(Vec<Ident>),
    /// Multi-part identifier, e.g. `table_alias.column` or `schema.table.col`
    CompoundIdentifier(Vec<Ident>),

    /// Nested expression e.g. `(foo > bar)` or `(1)`
    Nested(Box<Expr>),

    /// An exists expression `EXISTS(SELECT ...)`, used in expressions like
    /// `WHERE EXISTS (SELECT ...)`.
    Exists(Box<Query>),
    /// A parenthesized subquery `(SELECT ...)`, used in expression like
    /// `SELECT (subquery) AS x` or `WHERE (subquery) = x`
    Subquery(Box<Query>),

    /// `IS [NOT] NULL` operator
    IsNull(IsNullExpr),

    /// `IS [NOT] DISTINCT FROM` operator
    IsDistinctFrom(IsDistinctFromExpr),

    /// Unary operation e.g. `NOT foo`
    UnaryOp(UnaryOpExpr),
    /// Binary operation e.g. `1 + 1` or `foo > bar`
    BinaryOp(BinaryOpExpr),

    /// `<expr> [ NOT ] IN (val1, val2, ...)`
    InList(InListExpr),

    /// `<expr> [ NOT ] IN (SELECT ...)`
    InSubquery(InSubqueryExpr),

    /// `<expr> [ NOT ] BETWEEN <low> AND <high>`
    Between(BetweenExpr),

    /// `CASE [<operand>] WHEN <condition> THEN <result> ... [ELSE <result>] END`
    ///
    /// Note we only recognize a complete single expression as `<condition>`,
    /// not `< 0` nor `1, 2, 3` as allowed in a `<simple when clause>` per
    /// <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#simple-when-clause>
    Case(CaseExpr),

    /// `<expr> COLLATE collation`
    Collate(CollateExpr),

    /// CAST / TRY_CAST an expression to a different data type,
    /// e.g. `CAST(foo AS VARCHAR(123))`, `TRY_CAST(foo AS VARCHAR(123))`
    //  TRY_CAST differs from CAST in the choice of how to implement invalid conversions
    Cast(CastExpr),

    /// EXTRACT(DateTimeField FROM <expr>)
    Extract(ExtractExpr),

    /// SUBSTRING(<expr> [FROM <expr>] [FOR <expr>])
    Substring(SubstringExpr),

    /// TRIM([BOTH | LEADING | TRAILING] <expr> [FROM <expr>])\
    /// Or\
    /// TRIM(<expr>)
    Trim(TrimExpr),

    /// LISTAGG( [ DISTINCT ] <expr> [, <separator> ] [ON OVERFLOW <on_overflow>] ) )
    /// [ WITHIN GROUP (ORDER BY <within_group1>[, ...] ) ]
    ListAgg(ListAggExpr),

    /// Scalar function call e.g. `LEFT(foo, 5)`
    Function(Function),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(v) => write!(f, "{}", v),
            Self::TypedString { data_type, value } => {
                write!(f, "{} '{}'", data_type, escape_single_quote_string(value))
            }
            Self::Identifier(ident) => write!(f, "{}", ident),
            Self::Wildcard => f.write_str("*"),
            Self::QualifiedWildcard(idents) => write!(f, "{}.*", display_separated(idents, ".")),
            Self::CompoundIdentifier(idents) => write!(f, "{}", display_separated(idents, ".")),
            Self::Nested(expr) => write!(f, "({})", expr),
            Self::Exists(query) => write!(f, "EXISTS ({})", query),
            Self::Subquery(query) => write!(f, "({})", query),
            Self::IsNull(expr) => write!(f, "{}", expr),
            Self::IsDistinctFrom(expr) => write!(f, "{}", expr),
            Self::UnaryOp(expr) => write!(f, "{}", expr),
            Self::BinaryOp(expr) => write!(f, "{}", expr),
            Self::InList(expr) => write!(f, "{}", expr),
            Self::InSubquery(expr) => write!(f, "{}", expr),
            Self::Between(expr) => write!(f, "{}", expr),
            Self::Case(expr) => write!(f, "{}", expr),
            Self::Collate(expr) => write!(f, "{}", expr),
            Self::Cast(expr) => write!(f, "{}", expr),
            Self::Extract(expr) => write!(f, "{}", expr),
            Self::Substring(expr) => write!(f, "{}", expr),
            Self::Trim(expr) => write!(f, "{}", expr),
            Self::ListAgg(expr) => write!(f, "{}", expr),
            Self::Function(func) => write!(f, "{}", func),
        }
    }
}

/// `<expr> IS [NOT] NULL` operator.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IsNullExpr {
    pub negated: bool,
    pub expr: Box<Expr>,
}

impl fmt::Display for IsNullExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} IS {}NULL",
            self.expr,
            if self.negated { "NOT " } else { "" }
        )
    }
}

/// `<expr1> IS [NOT] DISTINCT FROM <expr2>` operator
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IsDistinctFromExpr {
    pub negated: bool,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl fmt::Display for IsDistinctFromExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} IS {}DISTINCT FROM {}",
            self.left,
            if self.negated { "NOT " } else { "" },
            self.right
        )
    }
}

/// Unary operation e.g. `NOT foo`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnaryOpExpr {
    op: UnaryOperator,
    expr: Box<Expr>,
}

impl fmt::Display for UnaryOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.op, self.expr)
    }
}

/// Binary operation e.g. `1 + 1` or `foo > bar`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BinaryOpExpr {
    pub op: BinaryOperator,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl fmt::Display for BinaryOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

/// `<expr> [ NOT ] IN (val1, val2, ...)`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InListExpr {
    pub expr: Box<Expr>,
    pub negated: bool,
    pub list: Vec<Expr>,
}

impl fmt::Display for InListExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}IN ({})",
            self.expr,
            if self.negated { "NOT " } else { "" },
            display_comma_separated(&self.list)
        )
    }
}

/// `<expr> [ NOT ] IN (SELECT ...)`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InSubqueryExpr {
    pub expr: Box<Expr>,
    pub negated: bool,
    pub subquery: Box<Query>,
}

impl fmt::Display for InSubqueryExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}IN ({})",
            self.expr,
            if self.negated { "NOT " } else { "" },
            self.subquery
        )
    }
}

/// `<expr> [ NOT ] BETWEEN <low> AND <high>`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BetweenExpr {
    pub expr: Box<Expr>,
    pub negated: bool,
    pub low: Box<Expr>,
    pub high: Box<Expr>,
}

impl fmt::Display for BetweenExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}BETWEEN {} AND {}",
            self.expr,
            if self.negated { "NOT " } else { "" },
            self.low,
            self.high
        )
    }
}

/// `CASE [<operand>] WHEN <condition> THEN <result> ... [ELSE <result>] END`
///
/// Note we only recognize a complete single expression as `<condition>`,
/// not `< 0` nor `1, 2, 3` as allowed in a `<simple when clause>` per
/// <https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#simple-when-clause>
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CaseExpr {
    pub operand: Option<Box<Expr>>,
    pub conditions: Vec<Expr>,
    pub results: Vec<Expr>,
    pub else_result: Option<Box<Expr>>,
}

impl fmt::Display for CaseExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CASE")?;
        if let Some(operand) = &self.operand {
            write!(f, " {}", operand)?;
        }
        for (c, r) in self.conditions.iter().zip(&self.results) {
            write!(f, " WHEN {} THEN {}", c, r)?;
        }
        if let Some(else_result) = &self.else_result {
            write!(f, " ELSE {}", else_result)?;
        }
        write!(f, " END")
    }
}

/// `<expr> COLLATE collation`
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CollateExpr {
    pub expr: Box<Expr>,
    pub collation: ObjectName,
}

impl fmt::Display for CollateExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} COLLATE {}", self.expr, self.collation)
    }
}

/// CAST / TRY_CAST an expression to a different data type,
/// e.g. `CAST(foo AS VARCHAR(123))`, `TRY_CAST(foo AS VARCHAR(123))`
//  TRY_CAST differs from CAST in the choice of how to implement invalid conversions
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CastExpr {
    pub r#try: bool,
    pub expr: Box<Expr>,
    pub data_type: DataType,
}

impl fmt::Display for CastExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.r#try {
            write!(f, "TRY_CAST({} AS {})", self.expr, self.data_type)
        } else {
            write!(f, "CAST({} AS {})", self.expr, self.data_type)
        }
    }
}

/// EXTRACT(DateTimeField FROM <expr>)
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExtractExpr {
    pub field: DateTimeField,
    pub expr: Box<Expr>,
}

impl fmt::Display for ExtractExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EXTRACT({} FROM {})", self.field, self.expr)
    }
}

/// SUBSTRING(<expr> [FROM <expr>] [FOR <expr>])
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubstringExpr {
    pub expr: Box<Expr>,
    pub substring_from: Option<Box<Expr>>,
    pub substring_for: Option<Box<Expr>>,
}

impl fmt::Display for SubstringExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SUBSTRING({}", self.expr)?;
        if let Some(from_part) = &self.substring_from {
            write!(f, " FROM {}", from_part)?;
        }
        if let Some(from_part) = &self.substring_for {
            write!(f, " FOR {}", from_part)?;
        }
        write!(f, ")")
    }
}

/// TRIM([BOTH | LEADING | TRAILING] <expr> [FROM <expr>])\
/// Or\
/// TRIM(<expr>)
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrimExpr {
    pub expr: Box<Expr>,
    // ([BOTH | LEADING | TRAILING], <expr>)
    pub trim_where: Option<(TrimWhereField, Box<Expr>)>,
}

impl fmt::Display for TrimExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TRIM(")?;
        if let Some((ident, trim_char)) = &self.trim_where {
            write!(f, "{} {} FROM {}", ident, trim_char, self.expr)?;
        } else {
            write!(f, "{}", self.expr)?;
        }
        write!(f, ")")
    }
}

/// [BOTH | LEADING | TRAILING]
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TrimWhereField {
    Both,
    Leading,
    Trailing,
}

impl fmt::Display for TrimWhereField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Both => "BOTH",
            Self::Leading => "LEADING",
            Self::Trailing => "TRAILING",
        })
    }
}

/// A `LISTAGG` invocation: LISTAGG( [ DISTINCT ] <expr> [, <separator> ] [ON OVERFLOW <on_overflow>] ) )
/// [ WITHIN GROUP (ORDER BY <within_group1>[, ...] ) ]
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ListAggExpr {
    pub distinct: bool,
    pub expr: Box<Expr>,
    pub separator: Option<Box<Expr>>,
    pub on_overflow: Option<ListAggOnOverflow>,
    pub within_group: Vec<OrderBy>,
}

impl fmt::Display for ListAggExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LISTAGG({}{}",
            if self.distinct { "DISTINCT " } else { "" },
            self.expr
        )?;
        if let Some(separator) = &self.separator {
            write!(f, ", {}", separator)?;
        }
        if let Some(on_overflow) = &self.on_overflow {
            write!(f, "{}", on_overflow)?;
        }
        write!(f, ")")?;
        if !self.within_group.is_empty() {
            write!(
                f,
                " WITHIN GROUP (ORDER BY {})",
                display_comma_separated(&self.within_group)
            )?;
        }
        Ok(())
    }
}

/// The `ON OVERFLOW` clause of a LISTAGG invocation
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ListAggOnOverflow {
    /// `ON OVERFLOW ERROR`
    Error,
    /// `ON OVERFLOW TRUNCATE [ <filler> ] WITH[OUT] COUNT`
    Truncate {
        filler: Option<Box<Expr>>,
        with_count: bool,
    },
}

impl fmt::Display for ListAggOnOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ON OVERFLOW")?;
        match self {
            ListAggOnOverflow::Error => write!(f, " ERROR"),
            ListAggOnOverflow::Truncate { filler, with_count } => {
                write!(f, " TRUNCATE")?;
                if let Some(filler) = filler {
                    write!(f, " {}", filler)?;
                }
                if *with_count {
                    write!(f, " WITH")?;
                } else {
                    write!(f, " WITHOUT")?;
                }
                write!(f, " COUNT")
            }
        }
    }
}
