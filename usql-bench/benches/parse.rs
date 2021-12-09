use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let query = "SELECT * FROM table1 WHERE id = 1";
    group.bench_function("sqlparser query1", |b| {
        use sqlparser::{tokenizer::Tokenizer, parser::Parser};
        let dialect = sqlparser::dialect::AnsiDialect {};
        let tokens = Tokenizer::new(&dialect, query).tokenize().unwrap();
        b.iter(|| {
            let _stmt = black_box(Parser::new( tokens.clone(), &dialect).parse_query().unwrap());
        });
    });
    group.bench_function("usql query1", |b| {
        use usql::{lexer::Lexer, parser::Parser};
        let dialect = usql::ansi::AnsiDialect::default();
        let tokens = Lexer::new(&dialect, query).tokenize().unwrap();
        b.iter(|| {
            let _stmt = black_box(Parser::new_with_tokens(&dialect, tokens.clone()).parse_select_stmt().unwrap());
        });
    });

    let query = "
        WITH derived AS (
            SELECT id1, id2
            FROM table1
            WHERE id1 > 100 AND id2 < 200
        )
        SELECT * FROM table1
        LEFT JOIN derived USING (id)
        ORDER BY id DESC
        OFFSET 20 ROWS
        FETCH FIRST 100 ROWS ONLY
        ";
    group.bench_function("sqlparser query2", |b| {
        use sqlparser::{tokenizer::Tokenizer, parser::Parser};
        let dialect = sqlparser::dialect::AnsiDialect {};
        let tokens = Tokenizer::new(&dialect, query).tokenize().unwrap();
        b.iter(|| {
            let _stmt = black_box(Parser::new( tokens.clone(), &dialect).parse_query().unwrap());
        });
    });
    group.bench_function("usql query2", |b| {
        use usql::{lexer::Lexer, parser::Parser};
        let dialect = usql::ansi::AnsiDialect::default();
        let tokens = Lexer::new(&dialect, query).tokenize().unwrap();
        b.iter(|| {
            let _stmt = black_box(Parser::new_with_tokens(&dialect, tokens.clone()).parse_select_stmt().unwrap());
        });
    });
}

criterion_group!(benches, parse);
criterion_main!(benches);
