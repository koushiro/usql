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
        use sqlparser::tokenizer::Tokenizer;
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let _tokens = black_box(Tokenizer::new(&dialect, input1).tokenize().unwrap());
        });
    });

    group.bench_function("usql-lexer 1", |b| {
        use usql::lexer::Lexer;
        let dialect = usql::core::ansi::AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, input1).tokenize().unwrap());
        });
    });

    group.bench_function("sqlparser 2", |b| {
        use sqlparser::tokenizer::Tokenizer;
        let dialect = sqlparser::dialect::AnsiDialect {};
        b.iter(|| {
            let _tokens = black_box(Tokenizer::new(&dialect, input2).tokenize().unwrap());
        });
    });

    group.bench_function("usql-lexer 2", |b| {
        use usql::lexer::Lexer;
        let dialect = usql::core::ansi::AnsiDialect::default();
        b.iter(|| {
            let _tokens = black_box(Lexer::new(&dialect, input2).tokenize().unwrap());
        });
    });
}

criterion_group!(benches, tokenize);
criterion_main!(benches);
