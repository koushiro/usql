#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

use crate::utils::escape_single_quote_string;

/// SQL literal values such as null, boolean, number, string, datetime and interval.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Literal {
    /// `NULL` value
    Null,

    /// Boolean literal, TRUE or FALSE
    Boolean(bool),

    /// Numeric literal
    Number(String),

    /// String literal (single quoted), e.g. 'string'
    String(String),
    /// National string literal, e.g. N'string'
    NationalString(String),
    /// Hex string literal, e.g. X'0123456789abcdef'
    HexString(String),
    /// Bit string literal, e.g. B'010101'
    BitString(String),

    /// DATE literal
    Date(Date),
    /// TIME literal
    Time(Time),
    /// TIMESTAMP literal
    Timestamp(Timestamp),

    /// INTERVAL literal
    Interval(Interval),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::Boolean(v) => {
                if *v {
                    f.write_str("TRUE")
                } else {
                    f.write_str("FALSE")
                }
            }
            Self::Number(v) => v.fmt(f),
            Self::String(v) => write!(f, "'{}'", escape_single_quote_string(v)),
            Self::NationalString(v) => write!(f, "N'{}'", v),
            Self::BitString(v) => write!(f, "B'{}'", v),
            Self::HexString(v) => write!(f, "X'{}'", v),
            Self::Date(v) => write!(f, "DATE '{}'", v),
            Self::Time(v) => write!(f, "TIME '{}'", v),
            Self::Timestamp(v) => write!(f, "TIMESTAMP '{}'", v),
            Self::Interval(v) => v.fmt(f),
        }
    }
}

/// Date literal, format: `DATE '<years>-<months>-<days>', e.g. `DATE '2021-11-09'`.
///
/// **NOTE**: the parser does not validate the `<value>` as required by the SQL specification.
/// Downstream consumers are responsible for rejecting date with invalid value.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
    /// The raw `<value>` that was present in `DATE '<value>'`.
    pub value: String,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Time literal, roughly in the following format:
///
/// ```txt
/// TIME '<hours>:<minutes>:<seconds> [ .<seconds fraction> ] [ <time zone interval>  ]'
/// ```
/// e.g. `TIME '11:40:12.1234+08:00'`.
///
/// **NOTE**: the parser does not validate the `<value>` as required by the SQL specification.
/// Downstream consumers are responsible for rejecting time with invalid value.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Time {
    /// The raw `<value>` that was present in `TIME '<value>'`.
    pub value: String,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Timestamp literal, roughly in the following format:
///
/// ```txt
/// TIMESTAMP '<years>-<months>-<days> <hours>:<minutes>:<seconds> [ .<seconds fraction> ] [ <time zone interval>  ]'
/// ```
///
/// **NOTE**: the parser does not validate the `<value>` as required by the SQL specification.
/// Downstream consumers are responsible for rejecting timestamp with invalid value.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp {
    /// The raw `<value>` that was present in `TIMESTAMP '<value>'`.
    pub value: String,
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// INTERVAL literals, roughly in the following format:
///
/// ```ignore
/// 1. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO <tailing field>
/// 2. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO SECOND [ (<fractional seconds precision>) ]
/// 3. INTERVAL '<value>' <leading field> [ (<leading precision>) ]
/// 4. INTERVAL '<value>' SECOND [ (<leading precision> [ , <fractional seconds precision> ] ) ]
/// ```
///
/// For example: `INTERVAL '123:45.67' MINUTE (3) TO SECOND (2)`
///
/// **Note**:
///
/// 1. The SQL standard allows an optional sign before the value string, but
/// it is not clear if any implementations support that syntax, so we
/// don't currently try to parse it. (The sign can instead be included
/// inside the value string.)
///
/// 2. The parser does not validate the `<value>`, nor does it ensure that the
/// `<leading_field>` units are coarser than the units in `<tailing_field>`,
/// as required by the SQL specification. Downstream consumers are responsible
/// for rejecting intervals with invalid values, like `'foobar'`, and invalid
/// unit specifications, like `HOUR TO YEAR`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interval {
    /// The raw `<value>` that was present in `INTERVAL '<value>'`.
    pub value: String,
    /// The unit of the first field in the interval.
    /// For example, `INTERVAL 'T' MINUTE` means `T` is in minutes.
    ///
    /// **Note**: PostgreSQL allows omitting the qualifier, so we provide
    /// this more general implementation.
    pub leading_field: Option<DateTimeField>,
    /// How many digits the leading field is allowed to occupy.
    ///
    /// Note that, according to the SQL specification, the interval `INTERVAL '1234' MINUTE(3)`
    /// is invalid, but `INTERVAL '123' MINUTE(3)` is valid.
    /// At present, such validation is left to downstream consumers.
    pub leading_precision: Option<u64>,
    /// How much precision to keep track of.
    ///
    /// If this is omitted, then clients should ignore all but the leading field.
    /// If it is less precise than the tailing field, clients should ignore the
    /// tailing field.
    ///
    /// For the following specifications:
    ///
    /// * in `INTERVAL '1:1:1' HOUR TO SECOND`, `tailing_field` will be
    ///   `Some(DateTimeField::Second)`, and clients should compute an
    ///   interval of 3661 seconds;
    /// * in `INTERVAL '1:1:1' HOUR TO MINUTE`, `tailing_field` will be
    ///   `Some(DateTimeField::Minute)`, and clients should compute an
    ///   interval of 3660 seconds;
    /// * in `INTERVAL '1:1:1' HOUR`, `tailing_field` will be `None`, and
    ///   clients should compute an interval of 3600 seconds.
    ///
    pub tailing_field: Option<DateTimeField>,
    /// If the tailing field is `SECOND`, the SQL standard permits the user to
    /// specify the fractional precision of the seconds. This specification can
    /// occur in either of two syntactic forms, depending on whether the
    /// interval's leading field is also `SECOND`.
    ///
    /// If both the leading and tailing fields are `SECOND`, then the fractional
    /// seconds precision is specified with the syntax `INTERVAL '_' SECOND (_, frac_prec)`.
    /// If only the tailing field is `SECOND`, then the fractional seconds precision
    /// is specified with the syntax `INTERVAL '_' {HOUR|MINUTE} TO SECOND (frac_prec)`.
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
                // the tailing field is None.
                assert!(self.tailing_field.is_none());
                write!(
                    f,
                    "INTERVAL '{}' SECOND({}, {})",
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
                    write!(f, "({})", leading_precision)?;
                }
                if let Some(tailing_field) = &self.tailing_field {
                    write!(f, " TO {}", tailing_field)?;
                }
                if let Some(fractional_seconds_precision) = &self.fractional_seconds_precision {
                    write!(f, "({})", fractional_seconds_precision)?;
                }
            }
        }
        Ok(())
    }
}

/// The leading/tailing field of interval.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_literal_display() {
        let string = Literal::String("hello".into());
        assert_eq!(string.to_string(), "'hello'");

        let national = Literal::NationalString("你好".into());
        assert_eq!(national.to_string(), "N'你好'");

        let bit = Literal::BitString("010101".into());
        assert_eq!(bit.to_string(), "B'010101'");

        let hex = Literal::HexString("1234567890abcdf".into());
        assert_eq!(hex.to_string(), "X'1234567890abcdf'");
    }

    #[test]
    fn datetime_literal_display() {
        let date = Date {
            value: "2021-11-29".into(),
        };
        assert_eq!(Literal::Date(date).to_string(), "DATE '2021-11-29'");

        let time = Time {
            value: "12:34:56".into(),
        };
        assert_eq!(Literal::Time(time).to_string(), "TIME '12:34:56'");
        let time = Time {
            value: "12:34:56.789".into(),
        };
        assert_eq!(Literal::Time(time).to_string(), "TIME '12:34:56.789'");
        let time = Time {
            value: "12:34:56.789+08:30".into(),
        };
        assert_eq!(Literal::Time(time).to_string(), "TIME '12:34:56.789+08:30'");

        let timestamp = Timestamp {
            value: "2021-11-29 12:34:56.789+08:30".into(),
        };
        assert_eq!(
            Literal::Timestamp(timestamp).to_string(),
            "TIMESTAMP '2021-11-29 12:34:56.789+08:30'"
        );
    }

    #[test]
    fn interval_literal_display() {
        let interval = Interval {
            value: "1-1".into(),
            leading_field: Some(DateTimeField::Year),
            leading_precision: None,
            tailing_field: Some(DateTimeField::Month),
            fractional_seconds_precision: None,
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1-1' YEAR TO MONTH"
        );

        let interval = Interval {
            value: "1:1:1.1".into(),
            leading_field: Some(DateTimeField::Hour),
            leading_precision: None,
            tailing_field: Some(DateTimeField::Second),
            fractional_seconds_precision: Some(5),
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1:1:1.1' HOUR TO SECOND(5)"
        );

        let interval = Interval {
            value: "1".into(),
            leading_field: Some(DateTimeField::Day),
            leading_precision: None,
            tailing_field: None,
            fractional_seconds_precision: None,
        };
        assert_eq!(Literal::Interval(interval).to_string(), "INTERVAL '1' DAY");

        let interval = Interval {
            value: "1.1".into(),
            leading_field: Some(DateTimeField::Second),
            leading_precision: Some(2),
            tailing_field: None,
            fractional_seconds_precision: Some(2),
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1.1' SECOND(2, 2)"
        );

        let interval = Interval {
            value: "1.1".into(),
            leading_field: Some(DateTimeField::Second),
            leading_precision: Some(2),
            tailing_field: None,
            fractional_seconds_precision: None,
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1.1' SECOND(2)"
        );

        let interval = Interval {
            value: "1.1".into(),
            leading_field: Some(DateTimeField::Second),
            leading_precision: None,
            tailing_field: None,
            fractional_seconds_precision: None,
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1.1' SECOND"
        );
    }
}
