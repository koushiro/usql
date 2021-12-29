use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn tokenize(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");

    let query = "SELECT * FROM table1 WHERE id = 1";
    group.bench_function("sqlparser query1", |b| {
        use sqlparser::tokenizer::Tokenizer;
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let _tokens = black_box(Tokenizer::new(&dialect, query).tokenize().unwrap());
        });
    });
    group.bench_function("usql query1", |b| {
        use usql::{ansi::AnsiDialect, Lexer};
        let dialect = AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, query).tokenize().unwrap());
        });
    });
    group.bench_function("usql-next query1", |b| {
        use usql_next::{AnsiDialect, Lexer};
        let dialect = AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, query).tokenize().unwrap());
        });
    });

    let query = "
        WITH derived AS (
            SELECT MAX(a) AS max_a,
               COUNT(b) AS b_num,
               id
            FROM table1
            GROUP BY id
        )
        SELECT * FROM table1
        LEFT JOIN derived USING (id)
        ";
    group.bench_function("sqlparser query2", |b| {
        use sqlparser::tokenizer::Tokenizer;
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let _tokens = black_box(Tokenizer::new(&dialect, query).tokenize().unwrap());
        });
    });
    group.bench_function("usql query2", |b| {
        use usql::{ansi::AnsiDialect, Lexer};
        let dialect = AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, query).tokenize().unwrap());
        });
    });
    group.bench_function("usql-next query2", |b| {
        use usql_next::{AnsiDialect, Lexer};
        let dialect = AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, query).tokenize().unwrap());
        });
    });
}

criterion_group!(benches, tokenize);
criterion_main!(benches);
