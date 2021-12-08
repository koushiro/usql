#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::ast::utils::display_separated;

/// An identifier, decomposed into its value or character data and the quote style.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ident {
    /// The value of the identifier without quotes.
    pub value: String,
    /// An identifier can be "quoted" (<delimited identifier> in ANSI parlance).
    /// The standard and most implementations allow using double quotes for this,
    /// but some implementations support other quoting styles as well.
    /// Valid quote characters are the single quote, double quote, backtick, and
    /// opening square bracket.
    pub quote: Option<char>,
}

impl Ident {
    /// Creates a new identifier with the given value and no quotes.
    pub fn new<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Ident {
            value: value.into(),
            quote: None,
        }
    }

    /// Creates a new quoted identifier with the given quote and value.
    /// This function panics if the given quote is not a valid quote character.
    pub fn with_quote<S>(quote: char, value: S) -> Self
    where
        S: Into<String>,
    {
        assert!(quote == '\'' || quote == '"' || quote == '`' || quote == '[');
        Ident {
            value: value.into(),
            quote: Some(quote),
        }
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Ident {
            value: value.into(),
            quote: None,
        }
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.quote {
            None => f.write_str(&self.value),
            Some(q) if q == '"' || q == '\'' || q == '`' => write!(f, "{}{}{}", q, self.value, q),
            Some(q) if q == '[' => write!(f, "[{}]", self.value),
            Some(q) => panic!("Unsupported quote character {} for SQL identifier!", q),
        }
    }
}

/// A name of a table, view, custom type, etc. (possibly multi-part, i.e. db.schema.obj)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectName(pub Vec<Ident>);

impl fmt::Display for ObjectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", display_separated(&self.0, "."))
    }
}

impl ObjectName {
    /// Creates a new object name from the given identifiers.
    pub fn new<T: IntoIterator<Item = S>, S: Into<String>>(parts: T) -> Self {
        ObjectName(parts.into_iter().map(|s| Ident::new(s)).collect())
    }
}
