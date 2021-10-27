#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{types::Ident, utils::display_separated};

/// A name of a table, view, custom type, etc., possibly multi-part, i.e. db.schema.obj
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ObjectName(pub Vec<Ident>);

impl fmt::Display for ObjectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", display_separated(&self.0, "."))
    }
}

///
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ObjectType {
    Table,
    View,
    Index,
    Schema,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            ObjectType::Table => "TABLE",
            ObjectType::View => "VIEW",
            ObjectType::Index => "INDEX",
            ObjectType::Schema => "SCHEMA",
        })
    }
}
