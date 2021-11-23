use usql_core::postgres::PostgresDialect;
use usql_lexer::{Lexer, LexerError};

fn main() -> Result<(), LexerError> {
    let dialect = PostgresDialect::default();
    // NOTE: `LIMIT` is not reserved keyword of ANSI SQL, but it's the reserved keyword of PostgreSQL.
    let input = "SELECT * FROM a WHERE id1 = 1 AND id2 = 2 ORDER BY something OFFSET 10 LIMIT 10";
    let tokens = Lexer::new(&dialect, input).tokenize()?;
    println!("{:#?}", tokens);
    Ok(())
}
