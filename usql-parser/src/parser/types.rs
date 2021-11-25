#[cfg(not(feature = "std"))]
use alloc::string::String;

use usql_ast::types::*;
use usql_core::Dialect;

use crate::{error::ParserError, parser::Parser};

impl<'a, D: Dialect> Parser<'a, D> {
    /// Parse identifier.
    pub fn parse_identifier(&mut self) -> Result<Ident, ParserError> {
        todo!()
    }

    /// Parse object name.
    pub fn parse_object_name(&mut self) -> Result<ObjectName, ParserError> {
        todo!()
    }

    /// Parse literal.
    pub fn parse_literal(&mut self) -> Result<Literal, ParserError> {
        todo!()
    }

    /// Parse unsigned number literal.
    pub fn parse_literal_uint(&mut self) -> Result<u64, ParserError> {
        todo!()
    }

    /// Parse string literal.
    pub fn parse_literal_string(&mut self) -> Result<String, ParserError> {
        todo!()
    }

    /// Parse date literal.
    pub fn parse_literal_date(&mut self) -> Result<Date, ParserError> {
        todo!()
    }

    /// Parse time literal.
    pub fn parse_literal_time(&mut self) -> Result<Time, ParserError> {
        todo!()
    }

    /// Parse timestamp literal.
    pub fn parse_literal_timestamp(&mut self) -> Result<Timestamp, ParserError> {
        todo!()
    }

    /// Parse interval literal.
    pub fn parse_literal_interval(&mut self) -> Result<Interval, ParserError> {
        todo!()
    }

    /// Parse date time field.
    #[allow(unused)]
    pub(crate) fn parse_date_time_field(&mut self) -> Result<DateTimeField, ParserError> {
        todo!()
    }

    /// Parse data type.
    pub fn parse_data_type(&mut self) -> Result<DataType, ParserError> {
        todo!()
    }

    #[allow(unused)]
    fn parse_optional_precision(&mut self) -> Result<Option<u64>, ParserError> {
        todo!()
    }

    #[allow(unused)]
    fn parse_optional_precision_scale(
        &mut self,
    ) -> Result<(Option<u64>, Option<u64>), ParserError> {
        todo!()
    }
}
