#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::utils::escape_single_quote_string;

/// Primitive SQL values such as number and string
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Value {
    /// `NULL` value
    Null,

    /// Boolean literal, true or false
    Boolean(bool),

    /// Numeric literal
    Number(String),

    /// Double quoted string literal, e.g. "string"
    DoubleQuotedString(String),

    /// Single quoted string literal, e.g. 'string'
    SingleQuotedString(String),

    /// National string literal, e.g. N'string'
    NationalString(String),
    /// Bit string literal, e.g. B'010101'
    BitString(String),
    /// Hex string literal, e.g. X'0123456789abcdef'
    HexString(String),

    /// INTERVAL literals
    Interval(Interval),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::DoubleQuotedString(v) => write!(f, "\"{}\"", v),
            Self::SingleQuotedString(v) => write!(f, "'{}'", escape_single_quote_string(v)),
            Self::NationalString(v) => write!(f, "N'{}'", v),
            Self::BitString(v) => write!(f, "B'{}'", v),
            Self::HexString(v) => write!(f, "X'{}'", v),
            Self::Interval(v) => write!(f, "{}", v),
        }
    }
}

/// INTERVAL literals, roughly in the following format:
/// `INTERVAL '<value>' [ <leading_field> [ (<leading_precision>) ] ]
/// [ TO <last_field> [ (<fractional_seconds_precision>) ] ]`,
/// e.g. `INTERVAL '123:45.67' MINUTE(3) TO SECOND(2)`.
///
/// The parser does not validate the `<value>`, nor does it ensure
/// that the `<leading_field>` units >= the units in `<last_field>`,
/// so the user will have to reject intervals like `HOUR TO YEAR`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Interval {
    pub value: String,
    pub leading_field: Option<DateTimeField>,
    pub leading_precision: Option<u64>,
    pub last_field: Option<DateTimeField>,
    /// The seconds precision can be specified in SQL source as
    /// `INTERVAL '__' SECOND(_, x)` (in which case the `leading_field`
    /// will be `Second` and the `last_field` will be `None`),
    /// or as `__ TO SECOND(x)`.
    pub fractional_seconds_precision: Option<u64>,
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (
            self.leading_field,
            self.leading_precision,
            self.fractional_seconds_precision,
        ) {
            (
                Some(DateTimeField::Second),
                Some(leading_precision),
                Some(fractional_seconds_precision),
            ) => {
                // When the leading field is SECOND, the parser guarantees that
                // the last field is None.
                assert!(self.last_field.is_none());
                write!(
                    f,
                    "INTERVAL '{}' SECOND ({}, {})",
                    escape_single_quote_string(&self.value),
                    leading_precision,
                    fractional_seconds_precision
                )?;
            }
            _ => {
                write!(f, "INTERVAL '{}'", escape_single_quote_string(&self.value))?;
                if let Some(leading_field) = &self.leading_field {
                    write!(f, " {}", leading_field)?;
                }
                if let Some(leading_precision) = &self.leading_precision {
                    write!(f, " ({})", leading_precision)?;
                }
                if let Some(last_field) = &self.last_field {
                    write!(f, " TO {}", last_field)?;
                }
                if let Some(fractional_seconds_precision) = &self.fractional_seconds_precision {
                    write!(f, " ({})", fractional_seconds_precision)?;
                }
            }
        }
        Ok(())
    }
}

///
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DateTimeField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl fmt::Display for DateTimeField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Year => "YEAR",
            Self::Month => "MONTH",
            Self::Day => "DAY",
            Self::Hour => "HOUR",
            Self::Minute => "MINUTE",
            Self::Second => "SECOND",
        })
    }
}
