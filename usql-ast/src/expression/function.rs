#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    expression::{Expr, OrderBy},
    types::{Ident, ObjectName},
    utils::display_comma_separated,
};

/// A function call
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Function {
    // aggregate functions may specify e.g. `COUNT(DISTINCT x)`
    pub distinct: bool,
    pub name: ObjectName,
    pub args: Vec<FunctionArg>,
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

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FunctionArg {
    Named { name: Ident, arg: Expr },
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

/// A window specification (i.e. `OVER (PARTITION BY .. ORDER BY .. etc.)`)
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WindowSpec {
    pub partition_by: Vec<Expr>,
    pub order_by: Vec<OrderBy>,
    pub window_frame: Option<WindowFrame>,
}

impl fmt::Display for WindowSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut delim = "";
        if !self.partition_by.is_empty() {
            delim = " ";
            write!(
                f,
                "PARTITION BY {}",
                display_comma_separated(&self.partition_by)
            )?;
        }
        if !self.order_by.is_empty() {
            f.write_str(delim)?;
            delim = " ";
            write!(f, "ORDER BY {}", display_comma_separated(&self.order_by))?;
        }
        if let Some(window_frame) = &self.window_frame {
            if let Some(end_bound) = &window_frame.end_bound {
                f.write_str(delim)?;
                write!(
                    f,
                    "{} BETWEEN {} AND {}",
                    window_frame.units, window_frame.start_bound, end_bound
                )?;
            } else {
                f.write_str(delim)?;
                write!(f, "{} {}", window_frame.units, window_frame.start_bound)?;
            }
        }
        Ok(())
    }
}

/// Specifies the data processed by a window function, e.g.
/// `RANGE UNBOUNDED PRECEDING` or `ROWS BETWEEN 5 PRECEDING AND CURRENT ROW`.
///
/// Note: The parser does not validate the specified bounds; the caller should
/// reject invalid bounds like `ROWS UNBOUNDED FOLLOWING` before execution.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WindowFrame {
    pub units: WindowFrameUnits,
    pub start_bound: WindowFrameBound,
    /// The right bound of the `BETWEEN .. AND` clause. The end bound of `None`
    /// indicates the shorthand form (e.g. `ROWS 1 PRECEDING`), which must
    /// behave the same as `end_bound = WindowFrameBound::CurrentRow`.
    pub end_bound: Option<WindowFrameBound>,
    // TBD: EXCLUDE
}

impl Default for WindowFrame {
    /// returns default value for window frame
    ///
    /// see https://www.sqlite.org/windowfunctions.html#frame_specifications
    fn default() -> Self {
        Self {
            units: WindowFrameUnits::Range,
            start_bound: WindowFrameBound::Preceding(None),
            end_bound: None,
        }
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WindowFrameBound {
    /// `CURRENT ROW`
    CurrentRow,
    /// `<N> PRECEDING` or `UNBOUNDED PRECEDING`
    Preceding(Option<u64>),
    /// `<N> FOLLOWING` or `UNBOUNDED FOLLOWING`.
    Following(Option<u64>),
}

impl fmt::Display for WindowFrameBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowFrameBound::CurrentRow => f.write_str("CURRENT ROW"),
            WindowFrameBound::Preceding(None) => f.write_str("UNBOUNDED PRECEDING"),
            WindowFrameBound::Following(None) => f.write_str("UNBOUNDED FOLLOWING"),
            WindowFrameBound::Preceding(Some(n)) => write!(f, "{} PRECEDING", n),
            WindowFrameBound::Following(Some(n)) => write!(f, "{} FOLLOWING", n),
        }
    }
}
