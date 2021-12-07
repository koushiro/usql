#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::{expression::*, types::*, utils::display_comma_separated};

/// The query specification, which is a restricted variant of `SELECT` statement
/// (without `WITH`/`ORDER BY`/`LIMIT`/`OFFSET`/`FETCH` clause), which may appear
/// either as the only body item of an `Query`, or as an operand to a set
/// operation like `UNION`.
///
/// ```txt
/// <query specification> ::= SELECT [ ALL | DISTINCT ] <select list> <table expression>
/// <select list> ::= * | <select sublist>  [ { ,  <select sublist>  }... ]
/// <table expression> ::= <from clause>
///     [ <where clause> ]
///     [ <group by clause> ]
///     [ <having clause> ]
///     [ <window clause> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuerySpec {
    /// Set quantifier, `ALL` or `DISTINCT`
    pub quantifier: Option<SetQuantifier>,
    /// projection expressions
    pub projection: Vec<SelectItem>,

    // <table expression>::= <from clause> [ <where clause> ] [ <group by clause> ] [ <having clause> ] [ <window clause> ]
    /// `FROM` clause
    pub from: From,
    /// `WHERE` clause
    pub r#where: Option<Where>,
    /// `GROUP BY` clause
    pub group_by: Option<GroupBy>,
    /// `HAVING` clause
    pub having: Option<Having>,
    /// `WINDOW` clause
    pub window: Option<Window>,
}

impl fmt::Display for QuerySpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("SELECT")?;
        if let Some(quantifier) = &self.quantifier {
            write!(f, " {}", quantifier)?;
        }
        write!(f, " {}", display_comma_separated(&self.projection))?;

        // table expression
        write!(f, " {}", self.from)?;
        if let Some(r#where) = &self.r#where {
            write!(f, " {}", r#where)?;
        }
        if let Some(group_by) = &self.group_by {
            write!(f, " {}", group_by)?;
        }
        if let Some(having) = &self.having {
            write!(f, " {}", having)?;
        }
        if let Some(window) = &self.window {
            write!(f, " {}", window)?;
        }
        Ok(())
    }
}

/// One item of the comma-separated list following `SELECT`.
///
/// ```txt
/// <select list> ::= * | <select sublist>  [ { ,  <select sublist>  }... ]
/// <select sublist> ::= <qualified asterisk> | <derived column>
/// <derived column> ::= <value expression>  [ AS <column name> ]
/// ```
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

// ============================================================================
// from clause
// ============================================================================

/// From clause.
///
/// ```txt
/// <from clause> ::= FROM <table reference list>
/// <table reference list> ::= <table reference> [ , ... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct From {
    /// The table reference list.
    pub list: Vec<TableReference>,
}

impl fmt::Display for From {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FROM {}", display_comma_separated(&self.list))
    }
}

/// A table reference.
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
///
/// See [table reference] for details.
///
/// [table references]: https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#_7_6_table_reference
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableReference {
    pub relation: TableFactor,
    pub joins: Vec<Join>,
}

impl fmt::Display for TableReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.relation)?;
        for join in &self.joins {
            write!(f, " {}", join)?;
        }
        Ok(())
    }
}

/// A table name or a parenthesized subquery with an optional alias
///
/// ```txt
/// <table factor> ::= <table or query name> | <derived table> | <parenthesized joined table>
/// ```
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
    NestedJoin(Box<TableReference>),
}

impl fmt::Display for TableFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Table { name, alias } => {
                write!(f, "{}", name)?;
                if let Some(alias) = alias {
                    write!(f, " {}", alias)?;
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
                    write!(f, " {}", alias)?;
                }
                Ok(())
            }
            Self::NestedJoin(table) => write!(f, "({})", table),
        }
    }
}

/// Table alias.
///
/// ```txt
/// <table alias> ::= AS <alias name> ( <columns> )
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableAlias {
    /// Alias name.
    pub name: Ident,
    /// Columns.
    pub columns: Option<Vec<Ident>>,
}

impl fmt::Display for TableAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AS {}", self.name)?;
        if let Some(columns) = &self.columns {
            write!(f, " ({})", display_comma_separated(columns))?;
        }
        Ok(())
    }
}

/// The `JOIN` relation.
///
/// ```txt
/// <cross join> ::= <table reference> CROSS JOIN <table factor>
/// <qualified join> ::= <table reference> [ <join type>  ] JOIN <table reference> <join specification>
/// <natural join> ::= <table reference> NATURAL [ <join type>  ] JOIN <table factor>
///
/// <join type> ::= INNER | { LEFT | RIGHT | FULL  [ OUTER ] }
/// <join specification> ::= ON <search condition> | USING ( <column name list> )
/// ```
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
            JoinOperator::InnerJoin(constraint) => {
                write!(f, "INNER JOIN {}{}", self.relation, constraint)
            }
            JoinOperator::LeftOuterJoin(constraint) => {
                write!(f, "LEFT JOIN {} {}", self.relation, constraint)
            }
            JoinOperator::RightOuterJoin(constraint) => {
                write!(f, "RIGHT JOIN {} {}", self.relation, constraint)
            }
            JoinOperator::FullOuterJoin(constraint) => {
                write!(f, "FULL JOIN {} {}", self.relation, constraint)
            }
            JoinOperator::NaturalInnerJoin => write!(f, "NATURAL INNER JOIN {}", self.relation),
            JoinOperator::NaturalLeftOuterJoin => write!(f, "NATURAL LEFT JOIN {}", self.relation,),
            JoinOperator::NaturalRightOuterJoin => {
                write!(f, "NATURAL RIGHT JOIN {}", self.relation,)
            }
            JoinOperator::NaturalFullOuterJoin => write!(f, "NATURAL FULL JOIN {}", self.relation,),
        }
    }
}

/// The join operator.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinOperator {
    CrossJoin,
    // default join if no join type is specified
    InnerJoin(JoinSpec),
    LeftOuterJoin(JoinSpec),
    RightOuterJoin(JoinSpec),
    FullOuterJoin(JoinSpec),
    // default natural join if no join type is specified
    NaturalInnerJoin,
    NaturalLeftOuterJoin,
    NaturalRightOuterJoin,
    NaturalFullOuterJoin,
}

/// The join specification.
///
/// ```txt
/// <join specification> ::= <join condition> | <named columns join>
/// <join condition> ::= ON <search condition>
/// <named columns join> ::= USING ( <join column list> )  [ AS <join correlation name>  ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinSpec {
    /// Join condition
    On(Box<Expr>),
    /// Named columns join
    Using {
        columns: Vec<Ident>,
        alias: Option<Ident>,
    },
}

impl fmt::Display for JoinSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::On(expr) => write!(f, " ON {}", expr),
            Self::Using { columns, alias } => {
                if let Some(alias) = alias {
                    write!(
                        f,
                        " USING ({}) AS {}",
                        display_comma_separated(columns),
                        alias
                    )
                } else {
                    write!(f, " USING ({})", display_comma_separated(columns))
                }
            }
        }
    }
}

// ============================================================================
// where clause
// ============================================================================

/// Where clause.
///
/// ```txt
/// <where clause> ::= WHERE <search condition>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Where {
    /// The search condition.
    pub expr: Box<Expr>,
}

impl fmt::Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WHERE {}", self.expr)
    }
}

// ============================================================================
// group by clause
// ============================================================================

/// Group by clause.
///
/// ```txt
/// <group by clause> ::= GROUP BY [ DISTINCT | ALL ] <group element> [ { , <group element> }... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupBy {
    /// Set quantifier.
    pub quantifier: Option<SetQuantifier>,
    /// The list of grouping element.
    pub list: Vec<GroupingElement>,
}

impl fmt::Display for GroupBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("GROUP BY ")?;
        if let Some(quantifier) = &self.quantifier {
            write!(f, "{} ", quantifier)?;
        }
        write!(f, "{}", display_comma_separated(&self.list))
    }
}

/// Grouping element.
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
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GroupingElement {
    Empty,
    OrdinarySet(GroupingSet),
    Rollup(Vec<GroupingSet>),
    Cube(Vec<GroupingSet>),
    Sets(Vec<GroupingElement>),
}

impl fmt::Display for GroupingElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("()"),
            Self::OrdinarySet(name) => write!(f, "{}", name),
            Self::Rollup(list) => write!(f, "ROLLUP ({})", display_comma_separated(list)),
            Self::Cube(list) => write!(f, "CUBE ({})", display_comma_separated(list)),
            Self::Sets(elements) => {
                write!(f, " GROUPING SETS ({})", display_comma_separated(elements))
            }
        }
    }
}

/// Ordinary grouping set, which is a kind of grouping element.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GroupingSet {
    /// grouping column reference
    Column(ObjectName),
    /// grouping column reference list
    Columns(Vec<ObjectName>),
}

impl fmt::Display for GroupingSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Column(name) => write!(f, "{}", name),
            Self::Columns(names) => write!(f, "({})", display_comma_separated(names)),
        }
    }
}

// ============================================================================
// having clause
// ============================================================================

/// Having clause.
///
/// ```txt
/// <having clause> ::= HAVING <search condition>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Having {
    /// The search condition.
    pub expr: Box<Expr>,
}

impl fmt::Display for Having {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HAVING {}", self.expr)
    }
}

// ============================================================================
// window clause
// ============================================================================

/// Window clause.
///
/// ```txt
/// <window clause> ::= WINDOW <window definition> [ { , <window definition> }... ]
/// ```
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
///
/// ```txt
/// <window definition> ::= <window name> [ AS ] <window specification>
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowDef {
    /// New window name.
    pub name: Ident,
    /// Window specification details.
    pub spec: WindowSpec,
}

impl fmt::Display for WindowDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} AS ({})", self.name, self.spec)
    }
}

/// Window specification details.
///
/// ```txt
/// <window specification> ::= ( <window specification details> )
/// <window specification details> ::= ( [<existing window name>] [ <window partition clause> ] [ <window order clause> ] [ <window frame clause> ] )
/// <window partition clause> ::= PARTITION BY <window partition column> [ { , <window partition column> }... ]
/// <window order clause> ::= ORDER BY { <sort_key> [ ASC | DESC ] [ NULLS FIRST | NULLS LAST ] } [, ...]`
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowSpec {
    /// The existing window name.
    pub name: Option<Ident>,
    /// Window partition clauses.
    pub partition_by: Option<Vec<ObjectName>>,
    /// Window order clauses.
    pub order_by: Option<OrderBy>,
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
        if let Some(partition_by) = &self.partition_by {
            f.write_str(delimit)?;
            delimit = " ";
            write!(f, "PARTITION BY {}", display_comma_separated(partition_by))?;
        }
        if let Some(order) = &self.order_by {
            f.write_str(delimit)?;
            delimit = " ";
            write!(f, "{}", order)?;
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
