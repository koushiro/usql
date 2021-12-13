#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::ast::{
    expression::*,
    types::*,
    utils::{display_comma_separated, display_separated},
};

// ============================================================================
// Table definition and manipulation
// ============================================================================

/// The `CREATE TABLE` statement.
///
/// ```txt
/// <table definition> ::=
///     CREATE [ <table scope> ] TABLE [ IF NOT EXISTS ] <table name> <table contents source>
///         [ WITH SYSTEM VERSIONING ]
///         [ ON COMMIT { PRESERVE | DELETE } ROWS ]
///
/// <table scope> ::= { GLOBAL | LOCAL } TEMPORARY
/// <table contents source> ::=
///     ( <table element> [, ...] )
///     | <typed table clause>
///     | [ ( <column> [, ...] ) ] AS ( <query expression> ) { WITH NO DATA | WITH DATA }
///
/// <table element> ::= <column definition> | <table constraint definition> | <like clause>
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTableStmt {
    /// Table scope.
    pub scope: Option<TableScope>,
    /// Flag indicates that check if the table does not exists.
    pub if_not_exists: bool,
    /// Table name.
    pub name: ObjectName,
    /// Table contents source.
    pub content: TableContent,
    pub on_commit: Option<OnCommit>,
}

impl fmt::Display for CreateTableStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CREATE")?;
        if let Some(scope) = &self.scope {
            write!(f, " {}", scope)?;
        }
        f.write_str(" TABLE")?;
        if self.if_not_exists {
            f.write_str(" IF NOT EXISTS")?;
        }
        write!(f, " {}", self.name)?;
        write!(f, " {}", self.content)?;
        if let Some(on_commit) = &self.on_commit {
            write!(f, " {}", on_commit)?;
        }
        Ok(())
    }
}

/// The scope of table definition.
///
/// ```txt
/// <table scope> ::= { GLOBAL | LOCAL } TEMPORARY
/// ```
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableScope {
    Local,
    Global,
}

impl fmt::Display for TableScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableScope::Local => write!(f, "LOCAL TEMPORARY"),
            TableScope::Global => write!(f, "GLOBAL TEMPORARY"),
        }
    }
}

/// The contents source of table definition.
///
/// ```txt
/// <table content> ::=
///     ( <column definition> [, ...] [, ] [ <table constraint definition> [, ...] ] )
///     | LIKE <table name> [ <like option> [, ...] ]
///     | AS { ( <query expression> ) | <query expression> }
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableContent {
    Definition {
        /// Columns.
        columns: Vec<ColumnDef>,
        /// Table constraints.
        constraints: Vec<TableConstraint>,
    },
    Like(TableLike),
    SubQuery(Box<Query>),
}

impl fmt::Display for TableContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Definition {
                columns,
                constraints,
            } => {
                f.write_str("(")?;
                write!(f, "{}", display_comma_separated(columns))?;
                if !constraints.is_empty() {
                    f.write_str(", ")?;
                }
                if !constraints.is_empty() {
                    write!(f, "{}", display_comma_separated(constraints))?;
                }
                f.write_str(")")
            }
            Self::Like(like) => write!(f, "{}", like),
            Self::SubQuery(query) => write!(f, "AS {}", query),
        }
    }
}

/// SQL constraint definition.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstraintDef<C> {
    /// Constraint name.
    pub name: Option<ObjectName>,
    /// Constraint kind.
    pub constraint: C,
}

impl<C: fmt::Display> fmt::Display for ConstraintDef<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            display_constraint_name(&self.name),
            self.constraint
        )
    }
}

fn display_constraint_name(name: &'_ Option<ObjectName>) -> impl fmt::Display + '_ {
    struct ConstraintName<'a>(&'a Option<ObjectName>);
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

/// SQL column definition.
///
/// ```txt
/// <column definition> ::= <column name> <data type> [ <column constraint definition> [, ...] ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColumnDef {
    /// Column name.
    pub name: Ident,
    /// Column type.
    pub data_type: DataType,
    /// Column constraints.
    pub constraints: Vec<ColumnConstraintDef>,
}

impl fmt::Display for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        for constraint in &self.constraints {
            write!(f, " {}", constraint)?;
        }
        Ok(())
    }
}

/// SQL column constraint definition.
///
/// ```txt
/// <column constraint definition> ::= [ CONSTRAINT <constraint name> ] <column constraint>
/// ```
pub type ColumnConstraintDef = ConstraintDef<ColumnConstraint>;

/// SQL column constraint kind.
///
/// ```txt
/// <column constraint> ::=
///     NULL
///     | NOT NULL
///     | <unique specification>
///     | <check constraint definition>
///     | <references specification>
///     | <default specification>
///     | <collation specification>
///
/// <unique specification> ::= UNIQUE | PRIMARY KEY
///
/// <check constraint definition> ::= CHECK ( <search condition> )
///
/// <references specification> ::= REFERENCES <table name> [ ( <column name> [, ...] ) ]
///     [ MATCH { FULL | PARTIAL | SIMPLE } ]
///     [ <referential triggered action> ]
/// <referential triggered action> ::= <update rule> [ <delete rule> ] | <delete rule> [ <update rule> ]
/// <update rule> ::= ON UPDATE <referential action>
/// <delete rule> ::= ON DELETE <referential action>
/// <referential action> ::= CASCADE | SET NULL | SET DEFAULT | RESTRICT | NO ACTION
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColumnConstraint {
    /// `NULL`
    Null,
    /// `NOT NULL`
    NotNull,
    /// Unique specification
    #[doc(hidden)]
    Unique { is_primary: bool },
    /// Check constraint definition
    Check(Box<Expr>),
    /// Referential specification
    References {
        /// Foreign table name.
        table: ObjectName,
        /// Referenced column list.
        referenced_columns: Option<Vec<Ident>>,
        /// Match type.
        match_type: Option<ReferentialMatchType>,
        /// `ON UPDATE` referential triggered action.
        on_update: Option<ReferentialAction>,
        /// `ON DELETE` referential triggered action.
        on_delete: Option<ReferentialAction>,
    },
    /// Default definition
    Default(Literal),
    /// Collation specification
    Collation(ObjectName),
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
            Self::Check(expr) => write!(f, "CHECK ({})", expr),
            Self::References {
                table,
                referenced_columns,
                match_type,
                on_update,
                on_delete,
            } => {
                write!(f, "REFERENCES {}", table)?;
                if let Some(referenced_columns) = referenced_columns {
                    write!(f, " ({})", display_comma_separated(referenced_columns))?;
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
            Self::Default(default) => write!(f, "DEFAULT {}", default),
            Self::Collation(collation) => write!(f, "COLLATE {}", collation),
        }
    }
}

/// SQL table constraint definition.
///
/// ```txt
/// <table constraint definition> ::= [ CONSTRAINT <constraint name> ] <table constraint>
/// ```
pub type TableConstraintDef = ConstraintDef<TableConstraint>;

/// SQL table constraint kind.
///
/// ```txt
/// <table constraint> ::=
///     <unique constraint definition>
///     | <check constraint definition>
///     | <referential constraint definition>
///
/// <unique constraint definition> ::= { UNIQUE | PRIMARY KEY } ( <column name> [, ...] )
///
/// <check constraint definition> ::= CHECK ( <search condition> )
///
/// <referential constraint definition> ::= FOREIGN KEY ( <column name> [, ...] ) <references specification>
/// <references specification> ::= REFERENCES <table name> [ ( <column name> [, ...] ) ]
///     [ MATCH { FULL | PARTIAL | SIMPLE } ]
///     [ <referential triggered action> ]
/// <referential triggered action> ::= <update rule> [ <delete rule> ] | <delete rule> [ <update rule> ]
/// <update rule> ::= ON UPDATE <referential action>
/// <delete rule> ::= ON DELETE <referential action>
/// <referential action> ::= CASCADE | SET NULL | SET DEFAULT | RESTRICT | NO ACTION
/// ``
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableConstraint {
    /// Unique constraint definition
    #[doc(hidden)]
    Unique {
        is_primary: bool,
        columns: Vec<Ident>,
    },
    /// Check constraint definition
    Check(Box<Expr>),
    /// Referential constraint definition
    ForeignKey {
        /// Referencing column list.
        referencing_columns: Vec<Ident>,
        /// Foreign table name.
        table: ObjectName,
        /// Referenced column list.
        referenced_columns: Option<Vec<Ident>>,
        /// Match type.
        match_type: Option<ReferentialMatchType>,
        /// `ON UPDATE` referential triggered action.
        on_update: Option<ReferentialAction>,
        /// `ON DELETE` referential triggered action.
        on_delete: Option<ReferentialAction>,
    },
}

impl fmt::Display for TableConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unique {
                is_primary,
                columns,
            } => write!(
                f,
                "{} ({})",
                if *is_primary { "PRIMARY KEY" } else { "UNIQUE" },
                display_comma_separated(columns)
            ),
            Self::Check(expr) => write!(f, "CHECK ({})", expr),
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
                if let Some(referenced_columns) = referenced_columns {
                    write!(f, " ({})", display_comma_separated(referenced_columns))?;
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

/// The `LIKE` clause is used in `CREATE TABLE` statement.
/// It specifies a table from which the new table automatically copies all
/// column names, their data types, and their not-null constraints.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableLike {
    /// Source table name.
    pub table: ObjectName,
    /// Like options.
    pub options: Option<Vec<LikeOption>>,
}

impl fmt::Display for TableLike {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LIKE {}", self.table)?;
        if let Some(options) = &self.options {
            write!(f, " {}", display_comma_separated(options))?;
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

/// On commit clause of `CREATE TABLE` statement.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OnCommit {
    PreserveRows,
    DeleteRows,
    Drop,
}

impl fmt::Display for OnCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PreserveRows => "PRESERVE ROWS",
            Self::DeleteRows => "DELETE ROWS",
            Self::Drop => "DROP",
        })
    }
}

/// The `ALTER TABLE` statement.
///
/// ```txt
/// <alter table statement> ::= ALTER TABLE <table name> [ IF EXISTS ] <alter table action>
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

/// The alter action of table.
///
/// ```txt
/// <alter table action> ::=
///     <add column definition>
///     | <alter column definition>
///     | <drop column definition>
///     | <add table constraint definition>
///     | <alter table constraint definition>
///     | <drop table constraint definition>
///     | <add table period definition>
///     | <drop table period definition>
///     | <add system versioning clause>
///     | <drop system versioning clause>
/// ```
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
        }
    }
}

// ============================================================================
// View definition and manipulation
// ============================================================================

/// The `CREATE VIEW` statement.
///
/// ```txt
/// <view definition> ::= CREATE [ RECURSIVE ] VIEW <table name> <view specification>
///     AS <query expression>  [ WITH [ CASCADED | LOCAL ] CHECK OPTION ]
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateViewStmt {
    /// Flag indicates that if a view of the same name already exists, the old one will be replaced.
    ///
    /// **NOTE: PostgreSQL/MySQL specific**
    pub or_replace: bool,
    /// Flag indicates that if the view is a recursive view.
    ///
    /// **NOTE: MySQL/SQLite not support**
    pub recursive: bool,
    /// Flag indicates that check if the view does not exists.
    ///
    /// **NOTE: SQLite specific**
    pub if_not_exists: bool,
    /// Viewed table name.
    pub name: ObjectName,
    /// Viewed columns.
    pub columns: Option<Vec<Ident>>,
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
            "CREATE {or_replace}{recursive} VIEW {if_not_exists}{view_name}",
            or_replace = if self.or_replace { "OR REPLACE " } else { "" },
            recursive = if self.recursive { "RECURSIVE " } else { "" },
            if_not_exists = if self.if_not_exists { "IF NOT EXISTS " } else { "" },
            view_name = self.name
        )?;
        if let Some(columns) = &self.columns {
            write!(f, " ({})", display_comma_separated(columns))?;
        }
        write!(f, " AS {}", self.query)?;
        if let Some(option) = &self.check_option {
            f.write_str(match option {
                ViewCheckOption::Cascaded => " WITH CASCADED CHECK OPTION",
                ViewCheckOption::Local => " WITH LOCAL CHECK OPTION",
                ViewCheckOption::None => " WITH CHECK OPTION",
            })?;
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
    None,
}

impl fmt::Display for ViewCheckOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cascaded => "CASCADED",
            Self::Local => "LOCAL",
            Self::None => "",
        })
    }
}

// ============================================================================
// Domain definition and manipulation
// ============================================================================

/// The `CREATE DOMAIN` statement.
///
/// ```txt
/// <domain definition> ::= CREATE DOMAIN <domain name> [ AS ] <predefined type> [ <domain constraint> [, ...] ]
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
        Ok(())
    }
}

/// SQL domain constraint definition.
///
/// ```txt
/// <domain constraint> ::= [ CONSTRAINT <constraint name> ] <domain constraint definition>
/// ```
pub type DomainConstraintDef = ConstraintDef<DomainConstraint>;

/// SQL domain constraint kind.
///
/// ```txt
/// <domain constraint definition> ::=
///     NULL
///     | NOT NULL
///     | <check constraint definition>
///     | <default definition>
///     | <collation specification>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DomainConstraint {
    /// `NULL`
    Null,
    /// `NOT NULL`
    NotNull,
    /// `CHECK (<search condition>)`
    Check(Box<Expr>),
    /// `DEFAULT <literal>`
    Default(Literal),
    /// `COLLATE <collation name>`
    Collation(ObjectName),
}

impl fmt::Display for DomainConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("NULL"),
            Self::NotNull => f.write_str("NOT NULL"),
            Self::Default(default) => write!(f, "DEFAULT {}", default),
            Self::Collation(collation) => write!(f, "COLLATE {}", collation),
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
    SetDefault(Literal),
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
/// <user-defined type definition> ::= CREATE TYPE <user-defined type body>
///
/// <user-defined type body> ::= <user-defined type name>
///     [ UNDER <super type name> ]
///     [ AS <representation> ]
///     [ <type option> [, ...] ]
///     [ <method specification> [, ...] ]
///
/// // Not support now
/// <method specification> ::= <original method specification> | <overriding method specification>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTypeStmt {
    /// Type name.
    pub name: ObjectName,
    /// Super type name.
    pub super_name: Option<ObjectName>,
    /// Type representation.
    pub representation: Option<TypeRepresentation>,
    /// Type options.
    pub options: Option<Vec<TypeOption>>,
}

impl fmt::Display for CreateTypeStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CREATE TYPE {}", self.name)?;
        if let Some(super_type) = &self.super_name {
            write!(f, " UNDER {}", super_type)?;
        }
        if let Some(representation) = &self.representation {
            write!(f, " AS {}", representation)?;
        }
        Ok(())
    }
}

/// The representation of user-defined type
///
/// ```txt
/// <representation> ::= <predefined type> | <collection type> | ( <attribution definition> [, ...] )
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeRepresentation {
    DataType(DataType),
    Attributes(Vec<TypeAttributeDef>),
}

impl fmt::Display for TypeRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataType(ty) => write!(f, "{}", ty),
            Self::Attributes(attrs) => write!(f, "{}", display_comma_separated(attrs)),
        }
    }
}

/// The user-defined type option.
///
/// ```txt
/// <type option> ::=
///     { INSTANTIABLE | NOT INSTANTIABLE }
///     | { FINAL | NOT FINAL }
///     | { REF USING <predefined type> | REF FROM ( <attribute name> [, ...] ) | REF IS SYSTEM GENERATED }
///     | CAST ( SOURCE AS REF ) WITH <cast to ref identifier>
///     | CAST ( REF AS SOURCE ) WITH <cast to type identifier>
///     | CAST ( SOURCE AS DISTINCT ) WITH <cast to distinct identifier>
///     | CAST ( DISTINCT AS SOURCE ) WITH <cast to source identifier>
/// ```
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeOption {
    Instantiable(bool),
    Final(bool),
    RefUsing(DataType),
    RefFrom(Vec<Ident>),
    RefIsSystemGenerated,
    CastToRef(Ident),
    CastToType(Ident),
    CastToDistinct(Ident),
    CastToSource(Ident),
}

impl fmt::Display for TypeOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instantiable(negated) => {
                write!(f, "{}INSTANTIABLE", if *negated { "NOT " } else { "" })
            }
            Self::Final(negated) => write!(f, "{}FINAL", if *negated { "NOT " } else { "" }),
            Self::RefUsing(ty) => write!(f, "REF USING {}", ty),
            Self::RefFrom(attrs) => write!(f, "REF FROM ({})", display_comma_separated(attrs)),
            Self::RefIsSystemGenerated => write!(f, "REF IS SYSTEM GENERATED"),
            Self::CastToRef(ident) => write!(f, "CAST ( SOURCE AS REF ) WITH {}", ident),
            Self::CastToType(ident) => write!(f, "CAST ( REF AS SOURCE ) WITH {}", ident),
            Self::CastToDistinct(ident) => write!(f, "CAST ( SOURCE AS DISTINCT ) WITH {}", ident),
            Self::CastToSource(ident) => write!(f, "CAST ( DISTINCT AS SOURCE ) WITH {}", ident),
        }
    }
}

/// The attribute definition of user-defined type.
///
/// ```txt
/// <attribute definition> ::= <attribute name> <data type> [ <default clause> ] [ <collate clause> ]
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAttributeDef {
    /// Attribute name.
    pub name: Ident,
    /// Data type.
    pub data_type: DataType,
    /// Default definition.
    pub default: Option<Literal>,
    /// Collation specification.
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
/// <alter type statement> ::= ALTER TYPE <user-defined type name> <alter type action>
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
///
/// ```txt
/// <alter type action> ::=
///     <add attribute definition>
///     | <drop attribute definition>
///     | <add original method specification>
///     | <add overriding method specification>
///     | <drop method specification>
/// ```
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
            Self::DropAttribute(name) => write!(f, "DROP ATTRIBUTE {} RESTRICT", name),
        }
    }
}

// ============================================================================
//  Drop manipulation of Schema/Table/View/Domain/Type/Index
// ============================================================================

/// The `DROP { SCHEMA | TABLE | VIEW | DOMAIN | TYPE | DATABASE | INDEX } <name> ...` statement
///
/// ```txt
/// <drop schema statement> ::= DROP SCHEMA <schema name> [ IF EXISTS ] <drop behavior>
/// <drop table statement> ::= DROP TABLE <table name> [ IF EXISTS ] <drop behavior>
/// <drop view statement> ::= DROP VIEW <table name> [ IF EXISTS ] <drop behavior>
/// <drop domain statement> ::= DROP DOMAIN <domain name> [ IF EXISTS ] <drop behavior>
/// <drop data type statement> ::= DROP TYPE <type name> [ IF EXISTS ] <drop behavior>
///
/// // Not ANSI SQL
/// <drop database statement> ::= DROP DATABASE <database name> [ IF EXISTS ] <drop behavior>
/// <drop index statement> ::= DROP INDEX <index name> [ IF EXISTS ] <drop behavior>
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropStmt {
    /// Object type.
    pub ty: ObjectType,
    /// Flag indicates that check if the `schema/table/view/domain/type/database/index` exists. (Non-standard)
    pub if_exists: bool,
    /// One or more object names to drop. (ANSI SQL requires exactly one)
    pub names: Vec<ObjectName>,
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
            object_name = display_comma_separated(&self.names),
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
    Domain,
    Type,
    Database,
    Index,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Schema => "SCHEMA",
            Self::Table => "TABLE",
            Self::View => "VIEW",
            Self::Domain => "DOMAIN",
            Self::Type => "TYPE",
            Self::Database => "DATABASE",
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
