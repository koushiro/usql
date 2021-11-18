mod keyword;

pub use self::keyword::MysqlKeyword;
use crate::dialect::{CustomDialect, DialectLexerConf, DialectParserConf};

/// The MySQL dialect.
pub type MysqlDialect = CustomDialect<MysqlKeyword, MySqlLexerConfig, MysqlParserConfig>;

/// The lexer configuration of MySQL dialect.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MySqlLexerConfig {
    /// If the ANSI_QUOTES SQL mode is enabled, it is also permissible to quote identifiers within
    /// double quotation marks.
    // Treat " as an identifier quote character (like the ` quote character) and not as a string
    // quote character. You can still use ` to quote identifiers with this mode enabled.
    // With ANSI_QUOTES enabled, you cannot use double quotation marks to quote literal strings
    // because they are interpreted as identifiers.
    pub ansi_quotes_mode: bool,
}

impl Default for MySqlLexerConfig {
    fn default() -> Self {
        Self {
            ansi_quotes_mode: true,
        }
    }
}

impl DialectLexerConf for MySqlLexerConfig {
    fn is_string_literal_quotation(&self, ch: char) -> bool {
        if self.ansi_quotes_mode {
            ch == '\''
        } else {
            ch == '"'
        }
    }

    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        if self.ansi_quotes_mode {
            ch == '"' || ch == '`'
        } else {
            ch == '`'
        }
    }

    // See https://dev.mysql.com/doc/refman/8.0/en/identifiers.html
    fn is_identifier_start(&self, ch: char) -> bool {
        // Identifiers may begin with a digit but unless quoted may not consist solely of digits,
        // but we don't support that, as that makes it hard to distinguish numeric literals.
        ch.is_ascii_alphabetic()
            || ch == '_'
            || ch == '$'
            || ('\u{0080}'..='\u{ffff}').contains(&ch)
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric()
            || ch == '_'
            || ch == '$'
            || ('\u{0080}'..='\u{ffff}').contains(&ch)
    }
}

/// The parser configuration of MySQL dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MysqlParserConfig {}

impl DialectParserConf for MysqlParserConfig {}
