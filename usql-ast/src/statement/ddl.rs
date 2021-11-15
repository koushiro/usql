#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::{
    expression::*,
    statement::Stmt,
    types::*,
    utils::{display_comma_separated, display_separated},
};

// ============================================================================
// Schema definition and manipulation
// ============================================================================

/// The `CREATE SCHEMA` statement.
///
/// ```txt
/// CREATE SCHEMA [ IF NOT EXISTS ]
///     [ <schema name> |  AUTHORIZATION <authorization> |  <schema name> AUTHORIZATION <authorization> ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateSchemaStmt {
    /// Flag indicates that check if the schema does not exists.
    ///
    /// **NOTE: PostgreSQL specific**
    pub if_not_exists: bool,
    /// Schema name.
    pub name: Option<ObjectName>,
    /// Authorization clause.
    pub authorization: Option<Ident>,
    /// Schema element defines an object to be created within the schema.
    pub elements: Vec<Stmt>,
}

impl fmt::Display for CreateSchemaStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CREATE SCHEMA")?;
        if self.if_not_exists {
            f.write_str(" IF NOT EXISTS")?;
        }
        if let Some(name) = &self.name {
            write!(f, " {}", name)?;
        }
        if let Some(authorization) = &self.authorization {
            write!(f, " AUTHORIZATION {}", authorization)?;
        }
        if !self.elements.is_empty() {
            write!(f, "{}", display_separated(&self.elements, "\n"))?;
        }
        Ok(())
    }
}

// ============================================================================
// Table definition and manipulation
// ============================================================================

/// The `CREATE TABLE` statement.
///
/// ```txt
/// CREATE [ TEMPORARY ] TABLE [ IF NOT EXISTS ] <table name> (
///
/// )
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTableStmt {
    /// Flag indicates that if the table is temporary.
    pub temporary: bool,
    /// Flag indicates that check if the table does not exists.
    pub if_not_exists: bool,
    /// Table name.
    pub name: ObjectName,
    /// Columns.
    pub columns: Vec<ColumnDef>,
    /// Table constraints.
    pub constraints: Vec<TableConstraintDef>,
    /// `LIKE` clause.
    pub like: Option<LikeClause>,
    /// `AS <query>` clause.
    pub query: Option<Box<Query>>,
}

impl fmt::Display for CreateTableStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CREATE {temporary}TABLE {if_not_exists}{table_name}",
            temporary = if self.temporary { "TEMPORARY " } else { "" },
            if_not_exists = if self.if_not_exists { "IF NOT EXISTS " } else { "" },
            table_name = self.name,
        )?;

        if let Some(like) = &self.like {
            write!(f, " LIKE {}", like)?;
        }
        if let Some(query) = &self.query {
            write!(f, " AS {}", query)?;
        }
        Ok(())
    }
}

/// SQL table constraint definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableConstraintDef {
    /// Table constraint name.
    pub name: Option<Ident>,
    /// Table constraint kind.
    pub constraint: TableConstraint,
}

impl fmt::Display for TableConstraintDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            display_constraint_name(&self.name),
            self.constraint
        )
    }
}

/// SQL table constraint kind.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableConstraint {
    /// `UNIQUE | PRIMARY KEY (<columns>)`
    #[doc(hidden)]
    Unique {
        columns: Vec<Ident>,
        is_primary: bool,
    },
    /// ```txt
    /// FOREIGN KEY (<referencing columns>) REFERENCES <table> [ (<referenced columns>) ]
    /// [
    ///     [ ON UPDATE <referential action> ] [ ON DELETE <referential action> ] |
    ///     [ ON DELETE <referential action> ] [ ON UPDATE <referential action> ]
    /// ]
    /// ```
    ForeignKey {
        /// Referencing column list.
        referencing_columns: Vec<Ident>,
        /// Foreign table name.
        table: ObjectName,
        /// Referenced column list.
        referenced_columns: Vec<Ident>,
        /// Match type.
        match_type: Option<ReferentialMatchType>,
        /// `ON UPDATE` referential triggered action.
        on_update: Option<ReferentialAction>,
        /// `ON DELETE` referential triggered action.
        on_delete: Option<ReferentialAction>,
    },
    /// `CHECK (<search condition>)`
    Check(Box<Expr>),
}

impl fmt::Display for TableConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unique {
                columns,
                is_primary,
            } => write!(
                f,
                "{} ({})",
                if *is_primary { "PRIMARY KEY" } else { "UNIQUE" },
                display_comma_separated(columns)
            ),
            Self::ForeignKey {
                referencing_columns,
                table,
                referenced_columns,
                match_type,
                on_update,
                on_delete,
            } => {
                write!(
                    f,
                    "FOREIGN KEY ({}) REFERENCES {}",
                    display_comma_separated(referencing_columns),
                    table,
                )?;
                if !referenced_columns.is_empty() {
                    write!(f, "({})", display_comma_separated(referenced_columns))?;
                }
                if let Some(match_type) = match_type {
                    write!(f, " MATCH {}", match_type)?;
                }
                if let Some(action) = on_update {
                    write!(f, " ON UPDATE {}", action)?;
                }
                if let Some(action) = on_delete {
                    write!(f, " ON DELETE {}", action)?;
                }
                Ok(())
            }
            Self::Check(expr) => write!(f, "CHECK ({})", expr),
        }
    }
}

/// SQL column definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColumnDef {
    /// Column name.
    pub name: Ident,
    /// Column type.
    pub data_type: DataType,
    /// Column constraints.
    pub constraints: Vec<ColumnConstraintDef>,
    /// Default clause.
    pub default: Option<Expr>,
    /// Collation name.
    pub collation: Option<ObjectName>,
}

impl fmt::Display for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        for constraint in &self.constraints {
            write!(f, " {}", constraint)?;
        }
        if let Some(default) = &self.default {
            write!(f, " DEFAULT {}", default)?;
        }
        if let Some(collation) = &self.collation {
            write!(f, " COLLATE {}", collation)?;
        }
        Ok(())
    }
}

/// SQL column constraint definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColumnConstraintDef {
    /// Column constraint name.
    pub name: Option<Ident>,
    /// Column constraint kind.
    pub constraint: ColumnConstraint,
}

impl fmt::Display for ColumnConstraintDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            display_constraint_name(&self.name),
            self.constraint
        )
    }
}

/// SQL column constraint kind.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColumnConstraint {
    /// `NULL`
    Null,
    /// `NOT NULL`
    NotNull,
    /// `UNIQUE | PRIMARY KEY`
    #[doc(hidden)]
    Unique { is_primary: bool },
    /// ```txt
    /// REFERENCES <table> [ (<referenced columns>) ]
    /// [
    ///     [ ON UPDATE <referential action> ] [ ON DELETE <referential action> ] |
    ///     [ ON DELETE <referential action> ] [ ON UPDATE <referential action> ]
    /// ]
    /// ```
    References {
        /// Foreign table name.
        table: ObjectName,
        /// Referenced column list.
        referenced_columns: Vec<Ident>,
        /// Match type.
        match_type: Option<ReferentialMatchType>,
        /// `ON UPDATE` referential triggered action.
        on_update: Option<ReferentialAction>,
        /// `ON DELETE` referential triggered action.
        on_delete: Option<ReferentialAction>,
    },
    /// `CHECK (<search condition>)`
    Check(Box<Expr>),
}

impl fmt::Display for ColumnConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::NotNull => f.write_str("NOT NULL"),
            Self::Unique { is_primary } => {
                if *is_primary {
                    f.write_str("PRIMARY KEY")
                } else {
                    f.write_str("UNIQUE")
                }
            }
            Self::References {
                table,
                referenced_columns,
                match_type,
                on_update,
                on_delete,
            } => {
                write!(f, "REFERENCES {}", table)?;
                if !referenced_columns.is_empty() {
                    write!(f, "({})", display_comma_separated(referenced_columns))?;
                }
                if let Some(match_type) = match_type {
                    write!(f, " MATCH {}", match_type)?;
                }
                if let Some(action) = on_update {
                    write!(f, " ON UPDATE {}", action)?;
                }
                if let Some(action) = on_delete {
                    write!(f, " ON DELETE {}", action)?;
                }
                Ok(())
            }
            Self::Check(expr) => write!(f, "CHECK ({})", expr),
        }
    }
}

/// Used in references constraints.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReferentialMatchType {
    Full,
    Partial,
    Simple,
}

impl fmt::Display for ReferentialMatchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Full => "FULL",
            Self::Partial => "PARTIAL",
            Self::Simple => "SIMPLE",
        })
    }
}

/// Used in references constraints in `ON UPDATE` and `ON DELETE` rules.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReferentialAction {
    Cascade,
    Restrict,
    SetNull,
    SetDefault,
    NoAction,
}

impl fmt::Display for ReferentialAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
            Self::NoAction => "NO ACTION",
        })
    }
}

fn display_constraint_name(name: &'_ Option<Ident>) -> impl fmt::Display + '_ {
    struct ConstraintName<'a>(&'a Option<Ident>);
    impl<'a> fmt::Display for ConstraintName<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(name) = self.0 {
                write!(f, "CONSTRAINT {} ", name)?;
            }
            Ok(())
        }
    }
    ConstraintName(name)
}

/// The `LIKE` clause is used in `CREATE TABLE` statement.
/// It specifies a table from which the new table automatically copies all
/// column names, their data types, and their not-null constraints.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LikeClause {
    /// Source table name.
    pub table: ObjectName,
    /// Like options.
    pub options: Vec<LikeOption>,
}

impl fmt::Display for LikeClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.table)?;
        for option in &self.options {
            write!(f, " {}", option)?
        }
        Ok(())
    }
}

/// Like option of `LIKE` clause.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LikeOption {
    IncludingIdentity,
    ExcludingIdentity,
    IncludingDefaults,
    ExcludingDefaults,
    IncludingGenerated,
    ExcludingGenerated,
}

impl fmt::Display for LikeOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::IncludingIdentity => "INCLUDING IDENTITY",
            Self::ExcludingIdentity => "EXCLUDING IDENTITY",
            Self::IncludingDefaults => "INCLUDING DEFAULTS",
            Self::ExcludingDefaults => "EXCLUDING DEFAULTS",
            Self::IncludingGenerated => "INCLUDING GENERATED",
            Self::ExcludingGenerated => "EXCLUDING GENERATED",
        })
    }
}

/// The `ALTER TABLE` statement.
///
/// ```txt
/// ALTER TABLE <table name> <action>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlterTableStmt {
    /// Flag indicates that check if the table exists. (Non-standard)
    pub if_exists: bool,
    /// Table name.
    pub name: ObjectName,
    /// Alter action.
    pub action: AlterTableAction,
}

impl fmt::Display for AlterTableStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ALTER TABLE {if_exists}{table_name} {action}",
            if_exists = if self.if_exists { "IF EXISTS " } else { "" },
            table_name = self.name,
            action = self.action,
        )
    }
}

/// The alter action of `ALTER TABLE` statement.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlterTableAction {
    AddColumn {
        /// Flag indicates that check if the column does not exist. (Non-standard)
        if_not_exists: bool,
        /// Column definition.
        column: ColumnDef,
    },
    DropColumn {
        /// Flag indicates that check if the column exists. (Non-standard)
        if_exists: bool,
        /// Column name.
        name: Ident,
        /// Drop behavior.
        drop_behavior: Option<DropBehavior>,
    },
    AddTableConstraint {
        /// Constraint definition.
        constraint: TableConstraintDef,
    },
    DropTableConstraint {
        /// Flag indicates that check if the table constraint exists. (Non-standard)
        if_exists: bool,
        /// Constraint name.
        name: ObjectName,
        /// Drop behavior.
        drop_behavior: Option<DropBehavior>,
    },
}

impl fmt::Display for AlterTableAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddColumn {
                if_not_exists,
                column,
            } => write!(
                f,
                "ADD COLUMN {if_not_exists}{column}",
                if_not_exists = if *if_not_exists { "IF NOT EXISTS " } else { "" },
                column = column,
            ),
            Self::DropColumn {
                if_exists,
                name,
                drop_behavior,
            } => {
                write!(
                    f,
                    "DROP COLUMN {if_exists}{name}",
                    if_exists = if *if_exists { "IF EXISTS " } else { "" },
                    name = name,
                )?;
                if let Some(behavior) = drop_behavior {
                    write!(f, " {}", behavior)?;
                }
                Ok(())
            }
            Self::AddTableConstraint { constraint } => write!(f, "ADD {}", constraint),
            Self::DropTableConstraint {
                if_exists,
                name,
                drop_behavior,
            } => {
                write!(
                    f,
                    "DROP CONSTRAINT {if_exists}{name}",
                    if_exists = if *if_exists { "IF EXISTS " } else { "" },
                    name = name,
                )?;
                if let Some(behavior) = drop_behavior {
                    write!(f, " {}", behavior)?;
                }
                Ok(())
            }
        }
    }
}

// ============================================================================
// View definition and manipulation
// ============================================================================

/// The `CREATE VIEW` statement.
///
/// ```txt
/// CREATE [ OR REPLACE ] [ RECURSIVE ] VIEW [ IF NOT EXISTS ] <view name> [ (columns) ]
///     AS <query>
///     [ WITH [ CASCADED | LOCAL ] CHECK OPTION ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateViewStmt {
    /// Flag indicates that if a view of the same name already exists, the old one will be replaced.
    ///
    /// **NOTE: PostgreSQL/MySQL specific**
    pub or_replace: bool,
    /// Flag indicates that check if the view does not exists.
    ///
    /// **NOTE: SQLite specific**
    pub if_not_exists: bool,
    /// Flag indicates that if the view is a recursive view.
    ///
    /// **NOTE: MySQL/SQLite not support**
    pub recursive: bool,
    /// Viewed table name.
    pub name: ObjectName,
    /// Viewed columns.
    pub columns: Vec<Ident>,
    /// A SQL query that specifies what to view.
    pub query: Box<Query>,
    /// Check option.
    ///
    /// **NOTE: SQLite not support**
    pub check_option: Option<ViewCheckOption>,
}

impl fmt::Display for CreateViewStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CREATE {recursive}{or_replace} VIEW {if_not_exists}{view_name} ({columns}) AS {query}",
            recursive = if self.recursive { "RECURSIVE " } else { "" },
            or_replace = if self.or_replace { "OR REPLACE " } else { "" },
            if_not_exists = if self.if_not_exists { "IF NOT EXISTS " } else { "" },
            view_name = self.name,
            columns = display_comma_separated(&self.columns),
            query = self.query,
        )?;
        if let Some(option) = &self.check_option {
            write!(f, " WITH {} CHECK OPTION", option)?;
        }
        Ok(())
    }
}

/// This option controls the behavior of automatically updatable views.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ViewCheckOption {
    Cascaded,
    Local,
}

impl fmt::Display for ViewCheckOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cascaded => "CASCADED",
            Self::Local => "LOCAL",
        })
    }
}

// ============================================================================
// Domain definition and manipulation
// ============================================================================

/// The `CREATE DOMAIN` statement.
///
/// ```txt
/// CREATE DOMAIN <domain name> [ AS ] <type>
///     [ { [ CONSTRAINT <name> ] NOT NULL | NULL | CHECK (expr) } ... ]
///     [ DEFAULT <default option> ]
///     [ COLLATE <collation name> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateDomainStmt {
    /// Domain name.
    pub name: ObjectName,
    /// Data type.
    pub data_type: DataType,
    /// Domain constraints.
    pub constraints: Vec<DomainConstraintDef>,
    /// Default clause.
    pub default: Option<Expr>,
    /// Collation name.
    pub collation: Option<ObjectName>,
}

impl fmt::Display for CreateDomainStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CREATE DOMAIN {domain_name} AS {data_type}",
            domain_name = self.name,
            data_type = self.data_type,
        )?;
        if !self.constraints.is_empty() {
            write!(f, " {}", display_separated(&self.constraints, " "))?;
        }
        if let Some(default) = &self.default {
            write!(f, " DEFAULT {}", default)?;
        }
        if let Some(collation) = &self.collation {
            write!(f, " COLLATE {}", collation)?;
        }
        Ok(())
    }
}

/// SQL domain constraint definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DomainConstraintDef {
    /// Domain constraint name.
    pub name: Option<Ident>,
    /// Domain constraint kind.
    pub constraint: DomainConstraint,
}

impl fmt::Display for DomainConstraintDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            display_constraint_name(&self.name),
            self.constraint
        )
    }
}

/// SQL domain constraint kind.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DomainConstraint {
    /// `NULL`
    Null,
    /// `NOT NULL`
    NotNull,
    /// `CHECK (<search condition>)`
    Check(Box<Expr>),
}

impl fmt::Display for DomainConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::NotNull => f.write_str("NOT NULL"),
            Self::Check(expr) => write!(f, "CHECK ({})", expr),
        }
    }
}

/// The `ALTER DOMAIN` statement.
///
/// ```txt
/// ALTER DOMAIN <domain name> <action>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlterDomainStmt {
    /// Domain name.
    pub name: ObjectName,
    /// Alter action.
    pub action: AlterDomainAction,
}

impl fmt::Display for AlterDomainStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ALTER DOMAIN {} {}", self.name, self.action)
    }
}

/// The alter action of `ALTER DOMAIN` statement.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlterDomainAction {
    SetDefault(Box<Expr>),
    DropDefault,
    AddConstraint(DomainConstraintDef),
    DropConstraint(Ident),
}

impl fmt::Display for AlterDomainAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetDefault(default) => write!(f, "SET DEFAULT {}", default),
            Self::DropDefault => f.write_str("DROP DEFAULT"),
            Self::AddConstraint(constraint) => write!(f, "ADD {}", constraint),
            Self::DropConstraint(name) => write!(f, "DROP CONSTRAINT {}", name),
        }
    }
}

// ============================================================================
// User-defined type definition and manipulation
// ============================================================================

/// The `CREATE TYPE` statement.
///
/// ```txt
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTypeStmt {
    /// Type name.
    pub name: ObjectName,
    /// Type definition.
    pub definition: Option<TypeDef>,
}

impl fmt::Display for CreateTypeStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CREATE TYPE {}", self.name)?;
        if let Some(def) = &self.definition {
            write!(f, " AS {}", def)?;
        }
        Ok(())
    }
}

/// The user-defined type definition.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeDef {
    DataType(DataType),
    MemberList(Vec<TypeAttributeDef>),
}

impl fmt::Display for TypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataType(ty) => write!(f, "{}", ty),
            Self::MemberList(attrs) => write!(f, "{}", display_comma_separated(attrs)),
        }
    }
}

/// The attribute definition of user-defined type.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAttributeDef {
    /// Attribute name.
    pub name: Ident,
    /// Data type.
    pub data_type: DataType,
    /// Default clause.
    pub default: Option<Expr>,
    /// Collation name.
    pub collation: Option<ObjectName>,
}

impl fmt::Display for TypeAttributeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        if let Some(default) = &self.default {
            write!(f, " DEFAULT {}", default)?;
        }
        if let Some(collation) = &self.collation {
            write!(f, " COLLATE {}", collation)?;
        }
        Ok(())
    }
}

/// The `ALTER TYPE` statement.
///
/// ```txt
/// ALTER TYPE <type name> <action>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlterTypeStmt {
    /// Type name.
    pub name: ObjectName,
    /// Alter action.
    pub action: AlterTypeAction,
}

impl fmt::Display for AlterTypeStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ALTER TYPE {} {}", self.name, self.action)
    }
}

/// The alter action of `ALTER TYPE` statement.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlterTypeAction {
    AddAttribute(TypeAttributeDef),
    DropAttribute(Ident),
}

impl fmt::Display for AlterTypeAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddAttribute(attr) => write!(f, "ADD ATTRIBUTE {}", attr),
            Self::DropAttribute(name) => write!(f, "DROP ATTRIBUTE {}", name),
        }
    }
}

// ============================================================================
// Index definition and manipulation
// ============================================================================

/// The `CREATE ... INDEX <index> ... ON <table> ...` statement.
///
/// ```txt
/// CREATE [ UNIQUE ] INDEX <index> [ IF NOT EXISTS ] ON <table>
///     [ { column [ ASC | DESC ] } ... ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateIndexStmt {
    /// Flag indicates that check if the index is unique.
    pub unique: bool,
    /// Flag indicates that check if the index does not exists.
    pub if_not_exists: bool,
    /// Index name.
    pub index: ObjectName,
    /// Table name.
    pub table: ObjectName,
    /// Indexed columns.
    pub columns: Vec<OrderBy>,
}

impl fmt::Display for CreateIndexStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CREATE {unique}INDEX {if_not_exists}{index_name} ON {table_name} ({columns})",
            unique = if self.unique { "UNIQUE " } else { "" },
            if_not_exists = if self.if_not_exists { "IF NOT EXISTS " } else { "" },
            index_name = self.index,
            table_name = self.table,
            columns = display_comma_separated(&self.columns),
        )
    }
}

// ============================================================================
//  Drop manipulation of Schema/Table/View/Domain/Type/Index
// ============================================================================

/// The `DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | INDEX } <name> ...` statement
///
/// ```txt
/// DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | INDEX }
///     [ IF EXISTS ] <index name>
///     [ CASCADE | RESTRICT ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropStmt {
    /// Flag indicates that check if the `schema/table/view/domain/type/index` exists. (Non-standard)
    pub if_exists: bool,
    /// Object type.
    pub ty: ObjectType,
    /// One or more object names to drop. (ANSI SQL requires exactly one)
    pub name: Vec<ObjectName>,
    /// Drop behavior.
    pub behavior: Option<DropBehavior>,
}

impl fmt::Display for DropStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DROP {object_type} {if_exists}{object_name}",
            object_type = self.ty,
            if_exists = if self.if_exists { "IF EXISTS " } else { "" },
            object_name = display_comma_separated(&self.name),
        )?;
        if let Some(behavior) = &self.behavior {
            write!(f, " {}", behavior)?;
        }
        Ok(())
    }
}

/// The object type of drop statement.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectType {
    Schema,
    Table,
    View,
    DOMAIN,
    Type,
    Index,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Schema => "SCHEMA",
            Self::Table => "TABLE",
            Self::View => "VIEW",
            Self::DOMAIN => "DOMAIN",
            Self::Type => "TYPE",
            Self::Index => "INDEX",
        })
    }
}

/// The behavior of drop statement.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DropBehavior {
    Cascade,
    Restrict,
}

impl fmt::Display for DropBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
        })
    }
}
