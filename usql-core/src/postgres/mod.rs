mod keyword;

pub use self::keyword::PostgresKeyword;
use crate::dialect::{Dialect, DialectLexerConf, DialectParserConf};

/// The PostgreSQL dialect.
#[derive(Clone, Debug, Default)]
pub struct PostgresDialect {
    /// PostgreSQL lexer configuration.
    pub lexer_conf: PostgresLexerConfig,
    /// PostgreSQL parser configuration.
    pub parser_conf: PostgresParserConfig,
}

impl Dialect for PostgresDialect {
    type Keyword = PostgresKeyword;
    type LexerConf = PostgresLexerConfig;
    type ParserConf = PostgresParserConfig;

    fn lexer_conf(&self) -> &Self::LexerConf {
        &self.lexer_conf
    }

    fn parser_conf(&self) -> &Self::ParserConf {
        &self.parser_conf
    }
}

/// The lexer configuration of PostgreSQL dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostgresLexerConfig {}

impl DialectLexerConf for PostgresLexerConfig {
    // See https://www.postgresql.org/docs/13/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
    //
    // SQL identifiers and key words must begin with a letter (a-z, but also letters with
    // diacritical marks and non-Latin letters) or an underscore (_).
    //
    // Subsequent characters in an identifier or key word can be letters, underscores, digits (0-9),
    // or dollar signs ($). Note that dollar signs are not allowed in identifiers according to the
    // letter of the SQL standard, so their use might render applications less portable.
    //
    // The SQL standard will not define a key word that contains digits or starts or ends with an
    // underscore, so identifiers of this form are safe against possible conflict with future
    // extensions of the standard.
    fn is_identifier_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }
}

/// The parser configuration of PostgreSQL dialect.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PostgresParserConfig {}

impl DialectParserConf for PostgresParserConfig {}
