#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec};

use crate::{
    ast::types::*,
    dialect::Dialect,
    error::{parse_error, ParserError},
    keywords::Keyword,
    parser::Parser,
    tokens::{Token, Word},
};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parses an identifier.
    pub fn parse_identifier(&mut self) -> Result<Ident, ParserError> {
        match self.next_token() {
            Some(Token::Word(w)) => Ok(Ident {
                quote: w.quote,
                value: w.value,
            }),
            unexpected => self.expected("identifier", unexpected),
        }
    }

    /// Parses an object name.
    pub fn parse_object_name(&mut self) -> Result<ObjectName, ParserError> {
        let mut idents = vec![];
        loop {
            idents.push(self.parse_identifier()?);
            if !self.next_token_if_is(&Token::Period) {
                break;
            }
        }
        Ok(ObjectName(idents))
    }

    /// Parses a literal.
    pub fn parse_literal(&mut self) -> Result<Literal, ParserError> {
        match self.next_token() {
            Some(Token::Word(w)) => match w.keyword {
                Some(Keyword::NULL) => Ok(Literal::Null),
                Some(Keyword::TRUE) => Ok(Literal::Boolean(true)),
                Some(Keyword::FALSE) => Ok(Literal::Boolean(false)),
                Some(Keyword::DATE) => Ok(Literal::Date(self.parse_literal_date()?)),
                Some(Keyword::TIME) => Ok(Literal::Time(self.parse_literal_time()?)),
                Some(Keyword::TIMESTAMP) => Ok(Literal::Timestamp(self.parse_literal_timestamp()?)),
                Some(Keyword::INTERVAL) => Ok(Literal::Interval(self.parse_literal_interval()?)),
                None if w.quote.is_some() => match w.quote {
                    Some('\'') => Ok(Literal::String(w.value)),
                    _ => self.expected("literal", Some(Token::Word(w))),
                },
                _ => self.expected("literal", Some(Token::Word(w))),
            },
            Some(Token::Number(n)) => Ok(Literal::Number(n)),
            Some(Token::String(s)) => Ok(Literal::String(s)),
            Some(Token::NationalString(s)) => Ok(Literal::NationalString(s)),
            Some(Token::HexString(s)) => Ok(Literal::HexString(s)),
            Some(Token::BitString(s)) => Ok(Literal::BitString(s)),
            unexpected => self.expected("literal", unexpected),
        }
    }

    /// Parses a date literal.
    pub fn parse_literal_date(&mut self) -> Result<Date, ParserError> {
        let value = self.parse_literal_string("date string")?;
        Ok(Date { value })
    }

    /// Parse a time literal.
    pub fn parse_literal_time(&mut self) -> Result<Time, ParserError> {
        let value = self.parse_literal_string("time string")?;
        Ok(Time { value })
    }

    /// Parses a timestamp literal.
    pub fn parse_literal_timestamp(&mut self) -> Result<Timestamp, ParserError> {
        let value = self.parse_literal_string("timestamp string")?;
        Ok(Timestamp { value })
    }

    /// Parses an interval literal.
    ///
    /// Some syntactically valid intervals:
    ///
    /// ```txt
    /// 1. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO <tailing field>
    /// 2. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO SECOND [ (<fractional seconds precision>) ]
    /// 3. INTERVAL '<value>' <leading field> [ (<leading precision>) ]
    /// 4. INTERVAL '<value>' SECOND [ (<leading precision> [ , <fractional seconds precision> ] ) ]
    /// ```
    ///
    /// Note: we do not currently attempt to parse the quoted value.
    pub fn parse_literal_interval(&mut self) -> Result<Interval, ParserError> {
        // The SQL standard allows an optional sign before the value string, but
        // it is not clear if any implementations support that syntax, so we
        // don't currently try to parse it. (The sign can instead be included
        // inside the value string.)

        // The first token in an interval is a string token which specifies
        // the duration of the interval.
        let value = self.parse_literal_string("interval string")?;

        // Following the string literal is a qualifier which indicates the units
        // of the duration specified in the string literal.
        //
        // Note that PostgreSQL allows omitting the qualifier, so we provide
        // this more general implementation.
        let leading_field = match self.peek_token() {
            Some(Token::Word(Word {
                keyword: Some(keyword),
                ..
            })) if [
                Keyword::YEAR,
                Keyword::MONTH,
                Keyword::DAY,
                Keyword::HOUR,
                Keyword::MINUTE,
                Keyword::SECOND,
            ]
            .into_iter()
            .any(|d| *keyword == d) =>
            {
                Some(self.parse_date_time_field()?)
            }
            _ => None,
        };

        let (leading_precision, tailing_field, fractional_seconds_precision) = if leading_field
            == Some(DateTimeField::Second)
        {
            // `SECOND [ (<leading precision> [ , <fractional seconds precision> ] ) ]`
            let (leading_precision, fractional_seconds_precision) =
                self.parse_optional_precision_scale()?;
            (leading_precision, None, fractional_seconds_precision)
        } else {
            let leading_precision = self.parse_optional_precision()?;
            if self.parse_keyword(Keyword::TO) {
                // `<leading field> [ (<leading precision>) ] TO <tailing field>`
                // `<leading field> [ (<leading precision>) ] TO SECOND [ (<fractional seconds precision>) ]`
                let tailing_field = Some(self.parse_date_time_field()?);
                let fractional_seconds_precision = if tailing_field == Some(DateTimeField::Second) {
                    self.parse_optional_precision()?
                } else {
                    None
                };
                (
                    leading_precision,
                    tailing_field,
                    fractional_seconds_precision,
                )
            } else {
                // `<leading field> [ (<leading precision>) ]`
                (leading_precision, None, None)
            }
        };
        Ok(Interval {
            value,
            leading_field,
            leading_precision,
            tailing_field,
            fractional_seconds_precision,
        })
    }

    /// Parses a data type.
    pub fn parse_data_type(&mut self) -> Result<DataType, ParserError> {
        // NOTE: we only support one-dimensional array
        let data_type = self.parse_simple_data_type()?;
        if self.parse_keyword(Keyword::ARRAY) {
            // ANSI SQL, e.g. INTEGER ARRAY, INTEGER ARRAY[10]
            if self.next_token_if_is(&Token::LeftBracket) {
                let length = self.parse_literal_uint()?;
                self.expect_token(&Token::RightBracket)?;
                Ok(DataType::Array(Box::new(data_type), Some(length)))
            } else {
                Ok(DataType::Array(Box::new(data_type), None))
            }
        } else if self.parse_keyword(Keyword::MULTISET) {
            // ANSI SQL, e.g. INTEGER MULTISET
            Ok(DataType::Multiset(Box::new(data_type)))
        } else {
            // PostgreSQL-specific array, e.g. INTEGER[], INTEGER[10]
            if self.next_token_if_is(&Token::LeftBracket) {
                if self.next_token_if_is(&Token::RightBracket) {
                    Ok(DataType::Array(Box::new(data_type), None))
                } else {
                    let length = self.parse_literal_uint()?;
                    self.expect_token(&Token::RightBracket)?;
                    Ok(DataType::Array(Box::new(data_type), Some(length)))
                }
            } else {
                Ok(data_type)
            }
        }
    }

    /// Parses a simple data type.
    pub fn parse_simple_data_type(&mut self) -> Result<DataType, ParserError> {
        match self.next_token() {
            Some(Token::Word(Word {
                keyword: Some(keyword),
                ..
            })) => match keyword {
                Keyword::BOOLEAN => Ok(DataType::Boolean),

                Keyword::TINYINT => Ok(DataType::TinyInt(self.parse_optional_precision()?)),
                Keyword::SMALLINT => Ok(DataType::SmallInt(self.parse_optional_precision()?)),
                Keyword::INT | Keyword::INTEGER => {
                    Ok(DataType::Int(self.parse_optional_precision()?))
                }
                Keyword::BIGINT => Ok(DataType::BigInt(self.parse_optional_precision()?)),

                Keyword::NUMERIC => {
                    let (precision, scale) = self.parse_optional_precision_scale()?;
                    Ok(DataType::Numeric { precision, scale })
                }
                Keyword::DECIMAL | Keyword::DEC => {
                    let (precision, scale) = self.parse_optional_precision_scale()?;
                    Ok(DataType::Decimal { precision, scale })
                }

                Keyword::FLOAT => Ok(DataType::Float(self.parse_optional_precision()?)),
                Keyword::REAL => Ok(DataType::Real),
                Keyword::DOUBLE => {
                    let _ = self.parse_keyword(Keyword::PRECISION);
                    Ok(DataType::Double)
                }

                Keyword::CHAR | Keyword::CHARACTER => {
                    if self.parse_keyword(Keyword::VARYING) {
                        Ok(DataType::Varchar(self.parse_precision()?))
                    } else {
                        Ok(DataType::Char(self.parse_optional_precision()?))
                    }
                }
                Keyword::VARCHAR => Ok(DataType::Varchar(self.parse_precision()?)),
                Keyword::CLOB => Ok(DataType::Clob(self.parse_optional_precision()?)),
                Keyword::TEXT => Ok(DataType::Text),

                Keyword::BINARY => {
                    if self.parse_keyword(Keyword::VARYING) {
                        Ok(DataType::Varbinary(self.parse_precision()?))
                    } else {
                        Ok(DataType::Binary(self.parse_optional_precision()?))
                    }
                }
                Keyword::VARBINARY => Ok(DataType::Varbinary(self.parse_precision()?)),
                Keyword::BLOB => Ok(DataType::Blob(self.parse_optional_precision()?)),
                Keyword::BYTEA => Ok(DataType::Bytea),

                Keyword::DATE => Ok(DataType::Date),
                Keyword::TIME => {
                    // TBD: we throw away "with/without timezone" information
                    if self.parse_keyword(Keyword::WITH) || self.parse_keyword(Keyword::WITHOUT) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                    }
                    Ok(DataType::Time)
                }
                Keyword::TIMESTAMP => {
                    // TBD: we throw away "with/without timezone" information
                    if self.parse_keyword(Keyword::WITH) || self.parse_keyword(Keyword::WITHOUT) {
                        self.expect_keywords(&[Keyword::TIME, Keyword::ZONE])?;
                    }
                    Ok(DataType::Timestamp)
                }
                // Interval types can be followed by a complicated interval qualifier that we don't currently support.
                // See parse_literal_interval for a taste.
                Keyword::INTERVAL => Ok(DataType::Interval),
                unexpected => self.expected("data type", Some(unexpected)),
            },
            Some(Token::Word(Word { keyword, .. })) if keyword.is_none() => {
                // TODO: custom types
                parse_error("Don't support custom data type yet")
            }
            unexpected => self.expected("data type", unexpected),
        }
    }

    fn parse_precision(&mut self) -> Result<u64, ParserError> {
        self.expect_token(&Token::LeftParen)?;
        let n = self.parse_literal_uint()?;
        self.expect_token(&Token::RightParen)?;
        Ok(n)
    }

    fn parse_optional_precision(&mut self) -> Result<Option<u64>, ParserError> {
        if self.next_token_if_is(&Token::LeftParen) {
            let n = self.parse_literal_uint()?;
            self.expect_token(&Token::RightParen)?;
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }

    fn parse_optional_precision_scale(
        &mut self,
    ) -> Result<(Option<u64>, Option<u64>), ParserError> {
        if self.next_token_if_is(&Token::LeftParen) {
            let precision = self.parse_literal_uint()?;
            let scale = if self.next_token_if_is(&Token::Comma) {
                Some(self.parse_literal_uint()?)
            } else {
                None
            };
            self.expect_token(&Token::RightParen)?;
            Ok((Some(precision), scale))
        } else {
            Ok((None, None))
        }
    }

    /// Parse unsigned number token.
    pub(crate) fn parse_literal_uint(&mut self) -> Result<u64, ParserError> {
        match self.next_token() {
            Some(Token::Number(n)) => n.parse::<u64>().map_err(|e| {
                ParserError::ParseError(format!("Could not parse '{}' as u64: {}", n, e))
            }),
            unexpected => self.expected("literal unsigned int", unexpected),
        }
    }

    /// Parses a string token.
    pub(crate) fn parse_literal_string(&mut self, expected: &str) -> Result<String, ParserError> {
        match self.next_token() {
            Some(Token::String(s)) => Ok(s),
            unexpected => self.expected(expected, unexpected),
        }
    }

    /// Parses a date time field.
    pub(crate) fn parse_date_time_field(&mut self) -> Result<DateTimeField, ParserError> {
        match self.next_token() {
            Some(Token::Word(w)) => match w.keyword {
                Some(Keyword::YEAR) => Ok(DateTimeField::Year),
                Some(Keyword::MONTH) => Ok(DateTimeField::Month),
                Some(Keyword::DAY) => Ok(DateTimeField::Day),
                Some(Keyword::HOUR) => Ok(DateTimeField::Hour),
                Some(Keyword::MINUTE) => Ok(DateTimeField::Minute),
                Some(Keyword::SECOND) => Ok(DateTimeField::Second),
                _ => self.expected("date/time field", Some(Token::Word(w))),
            },
            unexpected => self.expected("date/time field", unexpected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ident = Parser::new_with_sql(&dialect, "foo")?.parse_identifier()?;
        assert_eq!(ident, Ident::new("foo"));
        Ok(())
    }

    #[test]
    fn parse_object_name() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let name = Parser::new_with_sql(&dialect, "foo.bar")?.parse_object_name()?;
        assert_eq!(name, ObjectName(vec![Ident::new("foo"), Ident::new("bar")]));
        Ok(())
    }

    #[test]
    fn parse_literal() -> Result<(), ParserError> {
        pares_literal_boolean()?;
        parse_literal_number()?;
        parse_literal_string()?;
        parse_literal_date()?;
        parse_literal_time()?;
        parse_literal_timestamp()?;
        parse_literal_interval()?;
        Ok(())
    }

    // #[test]
    fn pares_literal_boolean() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let literal = Parser::new_with_sql(&dialect, "NULL")?.parse_literal()?;
        assert_eq!(literal, Literal::Null);
        let literal = Parser::new_with_sql(&dialect, "TRUE")?.parse_literal()?;
        assert_eq!(literal, Literal::Boolean(true));
        let literal = Parser::new_with_sql(&dialect, "FALSE")?.parse_literal()?;
        assert_eq!(literal, Literal::Boolean(false));
        Ok(())
    }

    // #[test]
    fn parse_literal_number() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let literal = Parser::new_with_sql(&dialect, "1234567890")?.parse_literal()?;
        assert_eq!(literal, Literal::Number("1234567890".into()));
        let literal = Parser::new_with_sql(&dialect, "1234567890.1234")?.parse_literal()?;
        assert_eq!(literal, Literal::Number("1234567890.1234".into()));
        Ok(())
    }

    // #[test]
    fn parse_literal_string() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let literal = Parser::new_with_sql(&dialect, "'foo'")?.parse_literal()?;
        assert_eq!(literal, Literal::String("foo".into()));
        let literal = Parser::new_with_sql(&dialect, "N'foo'")?.parse_literal()?;
        assert_eq!(literal, Literal::NationalString("foo".into()));
        let literal = Parser::new_with_sql(&dialect, "X'1234567890abcdef'")?.parse_literal()?;
        assert_eq!(literal, Literal::HexString("1234567890abcdef".into()));
        let literal = Parser::new_with_sql(&dialect, "B'10101010'")?.parse_literal()?;
        assert_eq!(literal, Literal::BitString("10101010".into()));
        Ok(())
    }

    // #[test]
    fn parse_literal_date() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "DATE '2021-11-29'")?.parse_literal()?,
            Literal::Date(Date {
                value: "2021-11-29".into()
            })
        );
        Ok(())
    }

    // #[test]
    fn parse_literal_time() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "TIME '12:34:56'")?.parse_literal()?,
            Literal::Time(Time {
                value: "12:34:56".into()
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "TIME '12:34:56.789'")?.parse_literal()?,
            Literal::Time(Time {
                value: "12:34:56.789".into()
            })
        );
        Ok(())
    }

    // #[test]
    fn parse_literal_timestamp() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "TIMESTAMP '2021-11-29 12:34:56.789+08:30'")?
                .parse_literal()?,
            Literal::Timestamp(Timestamp {
                value: "2021-11-29 12:34:56.789+08:30".into()
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "TIMESTAMP '2021-11-29 12:34:56+08:30'")?
                .parse_literal()?,
            Literal::Timestamp(Timestamp {
                value: "2021-11-29 12:34:56+08:30".into()
            })
        );
        Ok(())
    }

    // #[test]
    fn parse_literal_interval() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        // 1. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO <tailing field>
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1-1' YEAR TO MONTH")?.parse_literal()?,
            Literal::Interval(Interval {
                value: "1-1".into(),
                leading_field: Some(DateTimeField::Year),
                leading_precision: None,
                tailing_field: Some(DateTimeField::Month),
                fractional_seconds_precision: None,
            })
        );
        // 2. INTERVAL '<value>' <leading field> [ (<leading precision>) ] TO SECOND [ (<fractional seconds precision>) ]
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1:1:1.1' HOUR TO SECOND (5)")?
                .parse_literal()?,
            Literal::Interval(Interval {
                value: "1:1:1.1".into(),
                leading_field: Some(DateTimeField::Hour),
                leading_precision: None,
                tailing_field: Some(DateTimeField::Second),
                fractional_seconds_precision: Some(5),
            })
        );
        // 3. INTERVAL '<value>' <leading field> [ (<leading precision>) ]
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1' DAY")?.parse_literal()?,
            Literal::Interval(Interval {
                value: "1".into(),
                leading_field: Some(DateTimeField::Day),
                leading_precision: None,
                tailing_field: None,
                fractional_seconds_precision: None,
            })
        );
        // 4. INTERVAL '<value>' SECOND [ (<leading precision> [ , <fractional seconds precision> ] ) ]
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1.1' SECOND (2, 2)")?.parse_literal()?,
            Literal::Interval(Interval {
                value: "1.1".into(),
                leading_field: Some(DateTimeField::Second),
                leading_precision: Some(2),
                tailing_field: None,
                fractional_seconds_precision: Some(2),
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1.1' SECOND (2)")?.parse_literal()?,
            Literal::Interval(Interval {
                value: "1.1".into(),
                leading_field: Some(DateTimeField::Second),
                leading_precision: Some(2),
                tailing_field: None,
                fractional_seconds_precision: None,
            })
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTERVAL '1.1' SECOND")?.parse_literal()?,
            Literal::Interval(Interval {
                value: "1.1".into(),
                leading_field: Some(DateTimeField::Second),
                leading_precision: None,
                tailing_field: None,
                fractional_seconds_precision: None,
            })
        );
        Ok(())
    }

    #[test]
    fn parse_data_type() -> Result<(), ParserError> {
        parse_data_type_array()?;
        parse_data_type_multiset()?;
        parse_data_type_integer()?;
        parse_data_type_arbitrary_precision_number()?;
        parse_data_type_floating_point_number()?;
        parse_data_type_character_string()?;
        parse_data_type_binary_string()?;
        parse_data_type_datetime()?;
        Ok(())
    }

    // #[test]
    fn parse_data_type_array() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER ARRAY")?.parse_data_type()?,
            DataType::Array(Box::new(DataType::Int(None)), None)
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER ARRAY[10]")?.parse_data_type()?,
            DataType::Array(Box::new(DataType::Int(None)), Some(10))
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER[]")?.parse_data_type()?,
            DataType::Array(Box::new(DataType::Int(None)), None)
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER[10]")?.parse_data_type()?,
            DataType::Array(Box::new(DataType::Int(None)), Some(10))
        );
        Ok(())
    }

    // #[test]
    fn parse_data_type_multiset() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER MULTISET")?.parse_data_type()?,
            DataType::Multiset(Box::new(DataType::Int(None)))
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "INTEGER(10) MULTISET")?.parse_data_type()?,
            DataType::Multiset(Box::new(DataType::Int(Some(10))))
        );
        Ok(())
    }

    // #[test]
    fn parse_data_type_integer() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "BOOLEAN")?.parse_data_type()?;
        assert_eq!(ty, DataType::Boolean);

        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "SMALLINT")?.parse_data_type()?;
        assert_eq!(ty, DataType::SmallInt(None));
        let ty = Parser::new_with_sql(&dialect, "SMALLINT(5)")?.parse_data_type()?;
        assert_eq!(ty, DataType::SmallInt(Some(5)));
        let ty = Parser::new_with_sql(&dialect, "INT")?.parse_data_type()?;
        assert_eq!(ty, DataType::Int(None));
        let ty = Parser::new_with_sql(&dialect, "INT(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Int(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "INTEGER")?.parse_data_type()?;
        assert_eq!(ty, DataType::Int(None));
        let ty = Parser::new_with_sql(&dialect, "INTEGER(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Int(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "BIGINT")?.parse_data_type()?;
        assert_eq!(ty, DataType::BigInt(None));
        let ty = Parser::new_with_sql(&dialect, "BIGINT(19)")?.parse_data_type()?;
        assert_eq!(ty, DataType::BigInt(Some(19)));

        let dialect = crate::mysql::MysqlDialect::default();
        let ty = Parser::new_with_sql(&dialect, "TINYINT")?.parse_data_type()?;
        assert_eq!(ty, DataType::TinyInt(None));
        let ty = Parser::new_with_sql(&dialect, "TINYINT(3)")?.parse_data_type()?;
        assert_eq!(ty, DataType::TinyInt(Some(3)));
        Ok(())
    }

    // #[test]
    fn parse_data_type_arbitrary_precision_number() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        assert_eq!(
            Parser::new_with_sql(&dialect, "NUMERIC")?.parse_data_type()?,
            DataType::Numeric {
                precision: None,
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "NUMERIC(10)")?.parse_data_type()?,
            DataType::Numeric {
                precision: Some(10),
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "NUMERIC(10, 1)")?.parse_data_type()?,
            DataType::Numeric {
                precision: Some(10),
                scale: Some(1)
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DECIMAL")?.parse_data_type()?,
            DataType::Decimal {
                precision: None,
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DECIMAL(10)")?.parse_data_type()?,
            DataType::Decimal {
                precision: Some(10),
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DECIMAL(10, 1)")?.parse_data_type()?,
            DataType::Decimal {
                precision: Some(10),
                scale: Some(1)
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DEC")?.parse_data_type()?,
            DataType::Decimal {
                precision: None,
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DEC(10)")?.parse_data_type()?,
            DataType::Decimal {
                precision: Some(10),
                scale: None
            }
        );
        assert_eq!(
            Parser::new_with_sql(&dialect, "DEC(10, 1)")?.parse_data_type()?,
            DataType::Decimal {
                precision: Some(10),
                scale: Some(1)
            }
        );
        Ok(())
    }

    // #[test]
    fn parse_data_type_floating_point_number() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "FLOAT")?.parse_data_type()?;
        assert_eq!(ty, DataType::Float(None));
        let ty = Parser::new_with_sql(&dialect, "FLOAT(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Float(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "REAL")?.parse_data_type()?;
        assert_eq!(ty, DataType::Real);
        let ty = Parser::new_with_sql(&dialect, "DOUBLE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Double);
        let ty = Parser::new_with_sql(&dialect, "DOUBLE PRECISION")?.parse_data_type()?;
        assert_eq!(ty, DataType::Double);
        Ok(())
    }

    // #[test]
    fn parse_data_type_character_string() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "CHAR")?.parse_data_type()?;
        assert_eq!(ty, DataType::Char(None));
        let ty = Parser::new_with_sql(&dialect, "CHAR(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Char(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "CHARACTER")?.parse_data_type()?;
        assert_eq!(ty, DataType::Char(None));
        let ty = Parser::new_with_sql(&dialect, "CHARACTER(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Char(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "CHAR VARYING(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Varchar(10));
        let ty = Parser::new_with_sql(&dialect, "CHARACTER VARYING(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Varchar(10));
        let ty = Parser::new_with_sql(&dialect, "VARCHAR(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Varchar(10));
        let ty = Parser::new_with_sql(&dialect, "CLOB")?.parse_data_type()?;
        assert_eq!(ty, DataType::Clob(None));
        let ty = Parser::new_with_sql(&dialect, "CLOB(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Clob(Some(10)));

        let dialect = crate::postgres::PostgresDialect::default();
        let ty = Parser::new_with_sql(&dialect, "TEXT")?.parse_data_type()?;
        assert_eq!(ty, DataType::Text);
        Ok(())
    }

    // #[test]
    fn parse_data_type_binary_string() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "BINARY")?.parse_data_type()?;
        assert_eq!(ty, DataType::Binary(None));
        let ty = Parser::new_with_sql(&dialect, "BINARY(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Binary(Some(10)));
        let ty = Parser::new_with_sql(&dialect, "BINARY VARYING(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Varbinary(10));
        let ty = Parser::new_with_sql(&dialect, "VARBINARY(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Varbinary(10));
        let ty = Parser::new_with_sql(&dialect, "BLOB")?.parse_data_type()?;
        assert_eq!(ty, DataType::Blob(None));
        let ty = Parser::new_with_sql(&dialect, "BLOB(10)")?.parse_data_type()?;
        assert_eq!(ty, DataType::Blob(Some(10)));

        let dialect = crate::postgres::PostgresDialect::default();
        let ty = Parser::new_with_sql(&dialect, "BYTEA")?.parse_data_type()?;
        assert_eq!(ty, DataType::Bytea);
        Ok(())
    }

    // #[test]
    fn parse_data_type_datetime() -> Result<(), ParserError> {
        let dialect = crate::ansi::AnsiDialect::default();
        let ty = Parser::new_with_sql(&dialect, "DATE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Date);
        let ty = Parser::new_with_sql(&dialect, "TIME")?.parse_data_type()?;
        assert_eq!(ty, DataType::Time);
        let ty = Parser::new_with_sql(&dialect, "TIME WITH TIME ZONE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Time);
        let ty = Parser::new_with_sql(&dialect, "TIME WITHOUT TIME ZONE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Time);
        let ty = Parser::new_with_sql(&dialect, "TIMESTAMP")?.parse_data_type()?;
        assert_eq!(ty, DataType::Timestamp);
        let ty = Parser::new_with_sql(&dialect, "TIMESTAMP WITH TIME ZONE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Timestamp);
        let ty =
            Parser::new_with_sql(&dialect, "TIMESTAMP WITHOUT TIME ZONE")?.parse_data_type()?;
        assert_eq!(ty, DataType::Timestamp);
        let ty = Parser::new_with_sql(&dialect, "INTERVAL")?.parse_data_type()?;
        assert_eq!(ty, DataType::Interval);
        Ok(())
    }
}
