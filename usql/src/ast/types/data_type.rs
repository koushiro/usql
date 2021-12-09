#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use core::fmt;

use crate::ast::types::ObjectName;

/// SQL data types
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataType {
    /// Boolean
    Boolean,

    // ========================================================================
    // Integer Types
    // ========================================================================
    /// Tiny integer (-2^7 ~ 2^7 - 1) with optional display width e.g. TINYINT or TINYINT(3)
    TinyInt(Option<u64>),
    /// Small integer (-2^15 ~ 2^15 - 1) with optional display width e.g. SMALLINT or SMALLINT(5)
    SmallInt(Option<u64>),
    /// Integer (-2^31 ~ 2^31 - 1) with optional display width e.g. INT, INTEGER or INT(10), INTEGER(10)
    Int(Option<u64>),
    /// Big integer (-2^63 ~ 2^63 - 1) with optional display width e.g. BIGINT or BIGINT(19)
    BigInt(Option<u64>),

    // ========================================================================
    // Arbitrary Precision Numbers
    // ========================================================================
    /// Numeric type with optional precision and scale e.g. NUMERIC(10,2)
    Numeric {
        /// The total count of significant digits in the whole number
        precision: Option<u64>,
        /// The count of decimal digits in the fractional part
        scale: Option<u64>,
    },
    /// Decimal type with optional precision and scale e.g. DECIMAL(10,2)
    Decimal {
        /// The total count of significant digits in the whole number
        precision: Option<u64>,
        /// The count of decimal digits in the fractional part
        scale: Option<u64>,
    },

    // ========================================================================
    // Floating-Point Types
    // ========================================================================
    /// Floating point with optional precision e.g. FLOAT(8)
    Float(Option<u64>),
    /// Floating point e.g. REAL
    Real,
    /// Double e.g. DOUBLE PRECISION
    Double,

    // ========================================================================
    // Character String Types
    // ========================================================================
    /// Fixed-length character type e.g. CHAR(10)
    Char(Option<u64>),
    /// Variable-length character type e.g. VARCHAR(10)
    Varchar(u64),
    /// Character large object e.g. CLOB(1000)
    Clob(Option<u64>),
    /// Text type, variable unlimited length characters. (Not ANSI SQL)
    Text,

    // ========================================================================
    // Binary String Types
    // ========================================================================
    /// Fixed-length binary type e.g. BINARY(10)
    Binary(Option<u64>),
    /// Variable-length binary type e.g. VARBINARY(10)
    Varbinary(u64),
    /// Binary large object e.g. BLOB(1000)
    Blob(Option<u64>),
    /// Bytea type, variable-length binary string. (Not ANSI SQL)
    Bytea,

    // ========================================================================
    // Date/Time Types
    // ========================================================================
    /// Date
    Date,
    /// Time
    Time,
    /// Timestamp
    Timestamp,
    /// Interval
    Interval,

    // ========================================================================
    // Collection Types
    // ========================================================================
    /// Array
    Array(Box<DataType>, Option<u64>),
    /// Multiset
    Multiset(Box<DataType>),

    // ========================================================================
    // User-defined Types
    // ========================================================================
    /// User-defined type
    Custom(ObjectName),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Boolean => write!(f, "BOOLEAN"),

            DataType::TinyInt(zerofill) => format_type_with_optional_length(f, "TINYINT", zerofill),
            DataType::SmallInt(zerofill) => {
                format_type_with_optional_length(f, "SMALLINT", zerofill)
            }
            DataType::Int(zerofill) => format_type_with_optional_length(f, "INT", zerofill),
            DataType::BigInt(zerofill) => format_type_with_optional_length(f, "BIGINT", zerofill),

            DataType::Numeric { precision, scale } => {
                if let Some(scale) = scale {
                    write!(f, "NUMERIC({},{})", precision.unwrap(), scale)
                } else {
                    format_type_with_optional_length(f, "NUMERIC", precision)
                }
            }
            DataType::Decimal { precision, scale } => {
                if let Some(scale) = scale {
                    write!(f, "DECIMAL({},{})", precision.unwrap(), scale)
                } else {
                    format_type_with_optional_length(f, "DECIMAL", precision)
                }
            }

            DataType::Float(size) => format_type_with_optional_length(f, "FLOAT", size),
            DataType::Real => write!(f, "REAL"),
            DataType::Double => write!(f, "DOUBLE PRECISION"),

            DataType::Char(size) => format_type_with_optional_length(f, "CHAR", size),
            DataType::Varchar(size) => write!(f, "VARCHAR({})", size),
            DataType::Clob(size) => format_type_with_optional_length(f, "CLOB({})", size),
            DataType::Text => write!(f, "TEXT"),

            DataType::Binary(size) => format_type_with_optional_length(f, "BINARY({})", size),
            DataType::Varbinary(size) => write!(f, "VARBINARY({})", size),
            DataType::Blob(size) => format_type_with_optional_length(f, "BLOB({})", size),
            DataType::Bytea => write!(f, "BYTEA"),

            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Interval => write!(f, "INTERVAL"),

            DataType::Array(ty, length) => {
                if let Some(length) = length {
                    write!(f, "{}[{}]", ty, length)
                } else {
                    write!(f, "{}[]", ty)
                }
            }
            DataType::Multiset(ty) => write!(f, "{} MULTISET", ty),

            DataType::Custom(name) => write!(f, "{}", name),
        }
    }
}

fn format_type_with_optional_length(
    f: &mut fmt::Formatter,
    sql_type: &'static str,
    len: &Option<u64>,
) -> fmt::Result {
    write!(f, "{}", sql_type)?;
    if let Some(len) = len {
        write!(f, "({})", len)?;
    }
    Ok(())
}
