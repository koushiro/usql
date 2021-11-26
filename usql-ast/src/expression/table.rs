#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::{expression::*, types::*, utils::display_comma_separated};

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
    pub window: Option<Window>,
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
        if let Some(window) = &self.window {
            write!(f, " {}", window)?;
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

// ============================================================================
// window clause
// ============================================================================

/// Window clause.
///
/// ```txt
/// WINDOW <window_name> AS (<window_spec>) [, <window_name> AS (<window_spec>)] ...]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Window {
    /// The window definition list.
    pub list: Vec<WindowDef>,
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WINDOW {}", display_comma_separated(&self.list))
    }
}

/// Window definition.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowDef {
    /// New window name.
    pub name: Ident,
    /// Window specification.
    pub spec: WindowSpec,
}

impl fmt::Display for WindowDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} AS ({})", self.name, self.spec)
    }
}

/// Window specification.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowSpec {
    /// The existing window name.
    pub name: Option<Ident>,
    /// Window partition clauses.
    pub partition_by: Vec<Expr>,
    /// Window order clauses.
    pub order_by: Vec<OrderBy>,
    /// Window frame clause.
    pub window_frame: Option<WindowFrame>,
}

impl fmt::Display for WindowSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut delimit = "";
        if let Some(name) = &self.name {
            delimit = " ";
            write!(f, "{}", name)?;
        }
        if !self.partition_by.is_empty() {
            f.write_str(delimit)?;
            delimit = " ";
            write!(
                f,
                "PARTITION BY {}",
                display_comma_separated(&self.partition_by)
            )?;
        }
        if !self.order_by.is_empty() {
            f.write_str(delimit)?;
            delimit = " ";
            write!(f, "ORDER BY {}", display_comma_separated(&self.order_by))?;
        }
        if let Some(window_frame) = &self.window_frame {
            f.write_str(delimit)?;
            write!(f, "{}", window_frame)?;
        }
        Ok(())
    }
}

/// Specifies the data processed by a window function, e.g.
/// `RANGE UNBOUNDED PRECEDING` or `ROWS BETWEEN 5 PRECEDING AND CURRENT ROW`.
///
/// See https://www.sqlite.org/windowfunctions.html#frame_specifications for details.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowFrame {
    /// The frame type.
    pub units: WindowFrameUnits,
    /// The starting frame boundary.
    pub start_bound: WindowFrameBound,
    /// The ending frame boundary.
    /// The end bound of `Some` indicates the right bound of the `BETWEEN .. AND` clause.
    /// The end bound of `None` indicates the shorthand form (e.g. `ROWS 1 PRECEDING`),
    /// which must behave the same as `end_bound = WindowFrameBound::CurrentRow`.
    pub end_bound: Option<WindowFrameBound>,
    /// Exclude clause.
    pub exclusion: Option<WindowFrameExclusion>,
}

impl fmt::Display for WindowFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(end_bound) = &self.end_bound {
            write!(
                f,
                "{} BETWEEN {} AND {}",
                self.units, self.start_bound, end_bound
            )?;
        } else {
            write!(f, "{} {}", self.units, self.start_bound)?;
        }
        if let Some(exclusion) = self.exclusion {
            write!(f, " EXCLUDE {}", exclusion)?;
        }
        Ok(())
    }
}

/// The type of relationship between the current row and frame rows.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFrameUnits {
    Rows,
    Range,
    Groups,
}

impl fmt::Display for WindowFrameUnits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            WindowFrameUnits::Rows => "ROWS",
            WindowFrameUnits::Range => "RANGE",
            WindowFrameUnits::Groups => "GROUPS",
        })
    }
}

/// Specifies [WindowFrame]'s `start_bound` and `end_bound`
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFrameBound {
    /// `CURRENT ROW`.
    CurrentRow,
    /// `<N> PRECEDING` or `UNBOUNDED PRECEDING`.
    Preceding(Option<u64>),
    /// `<N> FOLLOWING` or `UNBOUNDED FOLLOWING`.
    Following(Option<u64>),
}

impl fmt::Display for WindowFrameBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowFrameBound::CurrentRow => f.write_str("CURRENT ROW"),
            WindowFrameBound::Preceding(None) => f.write_str("UNBOUNDED PRECEDING"),
            WindowFrameBound::Preceding(Some(n)) => write!(f, "{} PRECEDING", n),
            WindowFrameBound::Following(None) => f.write_str("UNBOUNDED FOLLOWING"),
            WindowFrameBound::Following(Some(n)) => write!(f, "{} FOLLOWING", n),
        }
    }
}

/// The exclude clause of window frame.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFrameExclusion {
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}

impl fmt::Display for WindowFrameExclusion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::CurrentRow => "CURRENT ROW",
            Self::Group => "GROUP",
            Self::Ties => "TIES",
            Self::NoOthers => "NO OTHERS",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_frame_display() {
        // With ORDER BY: The default frame includes rows from the partition
        // start through the current row, including all peers of the current row
        let frame = WindowFrame {
            units: WindowFrameUnits::Range,
            start_bound: WindowFrameBound::Preceding(None),
            end_bound: Some(WindowFrameBound::CurrentRow),
            exclusion: Some(WindowFrameExclusion::NoOthers),
        };
        assert_eq!(
            frame.to_string(),
            "RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW EXCLUDE NO OTHERS"
        );

        // Without ORDER BY: The default frame includes all partition rows
        // (because, without ORDER BY, all partition rows are peers)
        let frame = WindowFrame {
            units: WindowFrameUnits::Range,
            start_bound: WindowFrameBound::Preceding(None),
            end_bound: Some(WindowFrameBound::Following(None)),
            exclusion: Some(WindowFrameExclusion::NoOthers),
        };
        assert_eq!(
            frame.to_string(),
            "RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING EXCLUDE NO OTHERS"
        );

        let frame = WindowFrame {
            units: WindowFrameUnits::Range,
            start_bound: WindowFrameBound::Preceding(None),
            end_bound: None,
            exclusion: None,
        };
        assert_eq!(frame.to_string(), "RANGE UNBOUNDED PRECEDING");

        let frame = WindowFrame {
            units: WindowFrameUnits::Rows,
            start_bound: WindowFrameBound::Preceding(Some(5)),
            end_bound: Some(WindowFrameBound::CurrentRow),
            exclusion: None,
        };
        assert_eq!(
            frame.to_string(),
            "ROWS BETWEEN 5 PRECEDING AND CURRENT ROW"
        );
    }
}
