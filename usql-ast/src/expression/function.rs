#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

use crate::{
    expression::{Expr, OrderBy},
    types::{Ident, ObjectName},
    utils::display_comma_separated,
};

/// A function call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Function {
    // aggregate functions may specify e.g. `COUNT(DISTINCT x)`
    #[doc(hidden)]
    pub distinct: bool,
    /// The name of the function.
    pub name: ObjectName,
    /// The arguments of the function.
    pub args: Vec<FunctionArg>,
    /// The over clause.
    pub over: Option<WindowSpec>,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({}{})",
            self.name,
            if self.distinct { "DISTINCT " } else { "" },
            display_comma_separated(&self.args),
        )?;
        if let Some(o) = &self.over {
            write!(f, " OVER ({})", o)?;
        }
        Ok(())
    }
}

/// The arguments of a function call.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FunctionArg {
    /// Named argument.
    #[doc(hidden)]
    Named { name: Ident, arg: Expr },
    /// Unnamed argument.
    Unnamed(Expr),
}

impl fmt::Display for FunctionArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionArg::Named { name, arg } => write!(f, "{} => {}", name, arg),
            FunctionArg::Unnamed(unnamed_arg) => write!(f, "{}", unnamed_arg),
        }
    }
}

/// Window specification (i.e. `OVER (PARTITION BY .. ORDER BY .. etc.)`).
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
