/// The ANSI dialect.
#[cfg(feature = "ansi")]
pub mod ansi;
/// The MySQL dialect.
#[cfg(feature = "mysql")]
pub mod mysql;
/// The PostgreSQL dialect.
#[cfg(feature = "postgres")]
pub mod postgres;
/// The SQLite dialect.
#[cfg(feature = "sqlite")]
pub mod sqlite;
