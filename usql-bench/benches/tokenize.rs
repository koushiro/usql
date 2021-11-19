use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn tokenize(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");

    let input1 = "SELECT * FROM table WHERE 1 = 1";
    let input2 = "
        WITH derived AS (
            SELECT MAX(a) AS max_a,
               COUNT(b) AS b_num,
               user_id
            FROM TABLE
            GROUP BY user_id
        )
        SELECT * FROM table
        LEFT JOIN derived USING (user_id)
        ";

    group.bench_function("sqlparser 1", |b| {
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let mut tokenizer = sqlparser::tokenizer::Tokenizer::new(&dialect, input1);
            let _tokens = black_box(tokenizer.tokenize().unwrap());
        });
    });

    group.bench_function("usql-lexer 1", |b| {
        let dialect = usql_core::ansi::AnsiDialect::default();
        b.iter(|| {
            let mut tokenizer = usql_lexer::Lexer::new(input1, dialect.clone());
            let _tokens = black_box(tokenizer.tokenize().unwrap());
        });
    });

    group.bench_function("sqlparser 2", |b| {
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let mut tokenizer = sqlparser::tokenizer::Tokenizer::new(&dialect, input2);
            let _tokens = black_box(tokenizer.tokenize().unwrap());
        });
    });

    group.bench_function("usql-lexer 2", |b| {
        let dialect = usql_core::ansi::AnsiDialect::default();
        b.iter(|| {
            let mut tokenizer = usql_lexer::Lexer::new(input2, dialect.clone());
            let _tokens = black_box(tokenizer.tokenize().unwrap());
        });
    });
}

criterion_group!(benches, tokenize);
criterion_main!(benches);
