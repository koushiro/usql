#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::utils::escape_single_quote_string;

/// SQL literal values such as null, boolean, number, string, datetime and interval.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// Bit string literal, e.g. B'010101'
    BitString(String),
    /// Hex string literal, e.g. X'0123456789abcdef'
    HexString(String),

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
            Self::Number(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "'{}'", escape_single_quote_string(v)),
            Self::NationalString(v) => write!(f, "N'{}'", v),
            Self::BitString(v) => write!(f, "B'{}'", v),
            Self::HexString(v) => write!(f, "X'{}'", v),
            Self::Date(v) => write!(f, "DATE '{}'", v),
            Self::Time(v) => write!(f, "TIME '{}'", v),
            Self::Timestamp(v) => write!(f, "TIMESTAMP '{}'", v),
            Self::Interval(interval) => write!(f, "{}", interval),
        }
    }
}

/// Date literal, format: `DATE '<years>-<months>-<days>', e.g. `DATE '2021-11-09'`.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Date {
    pub years: u16, // u16::MAX = 65535, which is big enough for representing the year.
    pub months: u8,
    pub days: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.years, self.months, self.days)
    }
}

/// Time literal, roughly in the following format:
/// `TIME '<hours>:<minutes>:<seconds> [ .<seconds fraction> ] [ <time zone interval>  ]'`
/// e.g. `TIME '11:40:12.1234+08:00'`.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub seconds_fraction: Option<u32>,
    pub time_zone: Option<TimeZone>,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds
        )?;
        if let Some(seconds_fraction) = self.seconds_fraction {
            write!(f, ".{}", seconds_fraction)?;
        }
        if let Some(time_zone) = self.time_zone {
            write!(f, "{}", time_zone)?;
        }
        Ok(())
    }
}

/// The time zone field of time literal, format: `<sign><hours>:<minutes>`
/// e.g. `TIME '11:40:12.1234+08:00'`.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeZone {
    pub plus_sign: bool, // true: plus sign; false: minus sign.
    pub hours: u8,
    pub minutes: u8,
}

impl fmt::Display for TimeZone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.plus_sign {
            write!(f, "+{:02}:{:02}", self.hours, self.minutes)
        } else {
            write!(f, "-{:02}:{:02}", self.hours, self.minutes)
        }
    }
}

/// Timestamp literal, , roughly in the following format:
/// `TIMESTAMP '<years>-<months>-<days> <hours>:<minutes>:<seconds> [ .<seconds fraction> ] [ <time zone interval>  ]'`
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timestamp {
    pub date: Date,
    pub time: Option<Time>,
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(time) = self.time {
            write!(f, "{} {}", self.date, time)?;
        } else {
            write!(f, "{}", self.date)?;
        }
        Ok(())
    }
}

/// INTERVAL literals, roughly in the following format:
///
/// ```ignore
/// INTERVAL '<value>' <leading_field> [ (<leading_precision>) ]
///     [ TO <tailing_field> [ (<fractional_seconds_precision>) ] ]
/// ```
///
/// For example: `INTERVAL '123:45.67' MINUTE (3) TO SECOND (2)`
///
/// The parser does not validate the `<value>`, nor does it ensure that the
/// `<leading_field>` units are coarser than the units in `<tailing_field>`,
/// as required by the SQL specification. Downstream consumers are responsible
/// for rejecting intervals with invalid values, like `'foobar'`, and invalid
/// unit specifications, like `HOUR TO YEAR`.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Interval {
    /// The raw `<value>` that was present in `INTERVAL '<value>'`.
    pub value: String,
    /// The unit of the first field in the interval.
    /// For example, `INTERVAL 'T' MINUTE` means `T` is in minutes.
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
            years: 2021,
            months: 11,
            days: 9,
        };
        assert_eq!(Literal::Date(date).to_string(), "DATE '2021-11-09'");

        let mut time = Time {
            hours: 15,
            minutes: 37,
            seconds: 12,
            seconds_fraction: None,
            time_zone: None,
        };
        assert_eq!(Literal::Time(time).to_string(), "TIME '15:37:12'");
        time.seconds_fraction = Some(123456);
        assert_eq!(Literal::Time(time).to_string(), "TIME '15:37:12.123456'");
        time.time_zone = Some(TimeZone {
            plus_sign: true,
            hours: 8,
            minutes: 0,
        });
        assert_eq!(
            Literal::Time(time).to_string(),
            "TIME '15:37:12.123456+08:00'"
        );

        let timestamp = Timestamp {
            date,
            time: Some(time),
        };
        assert_eq!(
            Literal::Timestamp(timestamp).to_string(),
            "TIMESTAMP '2021-11-09 15:37:12.123456+08:00'"
        );
    }

    #[test]
    fn interval_literal_display() {
        let interval = Interval {
            value: "2021".to_string(),
            leading_field: Some(DateTimeField::Year),
            leading_precision: Some(4),
            tailing_field: None,
            fractional_seconds_precision: None,
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '2021' YEAR(4)"
        );

        let interval = Interval {
            value: "1:1:1".to_string(),
            leading_field: Some(DateTimeField::Second),
            leading_precision: Some(4),
            tailing_field: None,
            fractional_seconds_precision: Some(2),
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1:1:1' SECOND(4, 2)"
        );

        let interval = Interval {
            value: "1:1:1".to_string(),
            leading_field: Some(DateTimeField::Hour),
            leading_precision: None,
            tailing_field: Some(DateTimeField::Second),
            fractional_seconds_precision: Some(2),
        };
        assert_eq!(
            Literal::Interval(interval).to_string(),
            "INTERVAL '1:1:1' HOUR TO SECOND(2)"
        );
    }
}
