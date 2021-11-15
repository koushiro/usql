#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use core::fmt;

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
    /// Integer (-2^31 ~ 2^31 - 1) with optional display width e.g. INT or INT(10)
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
    // Character Types
    // ========================================================================
    /// Fixed-length character type e.g. CHAR(10)
    Char(Option<u64>),
    /// Variable-length character type e.g. VARCHAR(10)
    Varchar(Option<u64>),
    /// Character large object e.g. CLOB(1000)
    Clob(u64),
    /// Text type, variable unlimited length characters.
    Text,

    // ========================================================================
    // Binary Data Types
    // ========================================================================
    /// Fixed-length binary type e.g. BINARY(10)
    Binary(u64),
    /// Variable-length binary type e.g. VARBINARY(10)
    Varbinary(u64),
    /// Binary large object e.g. BLOB(1000)
    Blob(u64),
    /// Bytea type, variable-length binary string.
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
    Array(Box<DataType>),
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
            DataType::Varchar(size) => format_type_with_optional_length(f, "VARCHAR", size),
            DataType::Clob(size) => write!(f, "CLOB({})", size),
            DataType::Text => write!(f, "TEXT"),

            DataType::Binary(size) => write!(f, "BINARY({})", size),
            DataType::Varbinary(size) => write!(f, "VARBINARY({})", size),
            DataType::Blob(size) => write!(f, "BLOB({})", size),
            DataType::Bytea => write!(f, "BYTEA"),

            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Interval => write!(f, "INTERVAL"),

            DataType::Array(ty) => write!(f, "{}[]", ty),
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
