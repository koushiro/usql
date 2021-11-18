mod keyword;

pub use self::keyword::SqliteKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The SQLite dialect.
#[derive(Clone, Debug, Default)]
pub struct SqliteDialect {
    /// SQLite lexer configuration.
    pub lexer_conf: SqliteLexerConfig,
    /// SQLite parser configuration.
    pub parser_conf: SqliteParserConfig,
}

impl Dialect for SqliteDialect {
    type Keyword = SqliteKeyword;
    type LexerConf = SqliteLexerConfig;
    type ParserConf = SqliteParserConfig;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

/// The lexer configuration of SQLite dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SqliteLexerConfig {}

impl DialectLexerConf for SqliteLexerConfig {
    // See https://www.sqlite.org/lang_keywords.html
    //
    // A keyword enclosed in grave accents (ASCII code 96) is an identifier. This is not standard SQL.
    // This quoting mechanism is used by MySQL and is included in SQLite for compatibility.
    //
    // A keyword enclosed in square brackets is an identifier. This is not standard SQL.
    // This quoting mechanism is used by MS Access and SQL Server and is included in SQLite for compatibility.
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"' || ch == '`' || ch == '['
    }

    // See https://www.sqlite.org/draft/tokenreq.html
    //
    // ALPHABETIC: Any of the characters in the range u0041 through u005a (letters "A" through "Z")
    // or in the range u0061 through u007a (letters "a" through "z") or the character u005f ("_")
    // or any other character larger than u007f.
    //
    // SQLite shall recognize as an ID token any sequence of characters that begins with an
    // ALPHABETIC character and continue with zero or more ALPHANUMERIC characters
    // and/or "$" (u0024) characters and which is not a keyword token.
    fn is_identifier_start(&self, ch: char) -> bool {
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

/// The parser configuration of SQLite dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SqliteParserConfig {}

impl DialectParserConf for SqliteParserConfig {}
